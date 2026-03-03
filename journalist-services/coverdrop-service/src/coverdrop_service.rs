use anyhow::Result;
use chrono::{DateTime, Utc};
use common::{
    api::{
        api_client::ApiClient, forms::RotateJournalistIdPublicKeyFormForm,
        models::messages::user_to_journalist_message_with_dead_drop_id::UserToJournalistMessageWithDeadDropId,
    },
    client::VerifiedKeysAndJournalistProfiles,
    epoch::Epoch,
    identity_api::forms::post_rotate_journalist_id::RotateJournalistIdPublicKeyForm,
    protocol::{
        constants::{JOURNALIST_ID_KEY_ROTATE_AFTER, JOURNALIST_MSG_KEY_ROTATE_AFTER},
        covernode::verify_user_to_journalist_dead_drop_list,
        journalist::{
            encrypt_real_message_from_journalist_to_user_via_covernode,
            get_decrypted_journalist_dead_drop_message,
            new_encrypted_cover_message_from_journalist_via_covernode,
        },
        keys::{OrganizationPublicKeyFamilyList, UserPublicKey},
    },
    FixedSizeMessageText,
};
use journalist_vault::{JournalistVault, User};

use crate::constants::{JOURNALIST_ID_KEY_POLL_ITERATIONS, JOURNALIST_ID_KEY_POLL_SLEEP_DURATION};

pub enum ProcessVaultSetupBundleResult {
    AlreadyRegistered,
    SuccessfullyProcessedBundle,
}

pub struct JournalistCoverDropService {
    api_client: ApiClient,
    vault: JournalistVault,
}

impl JournalistCoverDropService {
    pub fn new(api_client: &ApiClient, vault: &JournalistVault) -> Self {
        Self {
            api_client: api_client.clone(),
            vault: vault.clone(),
        }
    }

    /// Pull dead drops from the API, verify them, decrypt messages, and store in vault.
    /// Returns the list of decrypted messages.
    ///
    /// The optional `on_progress` callback is called after each dead drop is processed,
    /// with the number of remaining dead drops to process.
    pub async fn pull_and_decrypt_dead_drops(
        &self,
        public_info: &VerifiedKeysAndJournalistProfiles,
        on_progress: Option<impl Fn(usize)>,
        now: DateTime<Utc>,
    ) -> Result<Vec<UserToJournalistMessageWithDeadDropId>> {
        let maybe_invoke_on_progress = |remaining: usize| {
            if let Some(ref callback) = on_progress {
                callback(remaining);
            }
        };
        let keys = &public_info.keys;

        let ids_greater_than = self.vault.max_dead_drop_id().await?;

        tracing::info!(
            "Pulling dead drops with ID greater than {}",
            ids_greater_than
        );

        let dead_drop_list = self
            .api_client
            .pull_all_journalist_dead_drops(ids_greater_than)
            .await?;

        let maybe_max_dead_drop_id = dead_drop_list
            .dead_drops
            .iter()
            .max_by_key(|d| d.id)
            .map(|d| d.id);
        let Some(max_dead_drop_id) = maybe_max_dead_drop_id else {
            tracing::info!("No dead drops in dead drop list");
            maybe_invoke_on_progress(0);
            return Ok(Vec::new());
        };

        // TODO should we return early if verified dead drops < total dead drops?
        // see https://github.com/guardian/coverdrop-internal/issues/3643
        let dead_drops = verify_user_to_journalist_dead_drop_list(keys, dead_drop_list, now);

        tracing::info!("Found {} dead drops", dead_drops.len());

        let total_dead_drops_to_process = dead_drops.len();

        // Notify progress with initial count
        maybe_invoke_on_progress(total_dead_drops_to_process);

        // find the max epoch in the list of dead drops to make sure that the public info epoch is high enough to decrypt
        let Some(max_dead_drop_epoch) = dead_drops.iter().max_by_key(|d| d.epoch).map(|d| d.epoch)
        else {
            // this check is redundant but necessary to turn the epoch into Some
            tracing::info!("No dead drops in dead drop list");
            // Callback was already called above with 0 since total_dead_drops_to_process would be 0
            return Ok(Vec::new());
        };

        if public_info.max_epoch < max_dead_drop_epoch {
            tracing::info!("Max epoch of public key hierarchy {} is less than the max dead drop epoch {}. Returning early.", public_info.max_epoch, max_dead_drop_epoch);
            maybe_invoke_on_progress(0);
            return Ok(Vec::new());
        } else {
            tracing::info!("Max epoch of public key hierarchy {} is greater than or equal to the max dead drop epoch {}. Attempting to decrypt.", public_info.max_epoch, max_dead_drop_epoch);
        }

        let journalist_msg_key_pairs = self
            .vault
            .msg_key_pairs_for_decryption(now)
            .await?
            .collect::<Vec<_>>();

        let covernode_msg_pks = keys
            .covernode_msg_pk_iter()
            .map(|(_, msg_pk)| msg_pk)
            .collect::<Vec<_>>();

        let decrypted_messages: Vec<UserToJournalistMessageWithDeadDropId> = dead_drops
            .iter()
            .enumerate()
            .flat_map(|(index, dead_drop)| {
                let processed_messages = dead_drop
                    .data
                    .messages
                    .iter()
                    .filter_map(|encrypted_message| {
                        get_decrypted_journalist_dead_drop_message(
                            &covernode_msg_pks,
                            &journalist_msg_key_pairs,
                            encrypted_message,
                            dead_drop.id,
                        )
                    })
                    .collect::<Vec<_>>();

                // Notify progress after processing each dead drop
                maybe_invoke_on_progress(total_dead_drops_to_process - index - 1);

                processed_messages
            })
            .collect();

        self.vault
            .add_messages_from_user_to_journalist_and_update_max_dead_drop_id(
                &decrypted_messages,
                max_dead_drop_id,
                now,
            )
            .await?;

        Ok(decrypted_messages)
    }

    /// Unconditionally rotate the journalist's identity key pair.
    /// Get or create a candidate key pair, upload it to the API, and promote to published if successful.
    /// The caller is responsible for checking if rotation is needed.
    ///
    /// This function will poll the API for up to 60 seconds to see if the key has been
    /// rotated. Returns Some(epoch) if rotation succeeded, None if it timed out.
    pub async fn rotate_id_key(&self, now: DateTime<Utc>) -> Result<Option<Epoch>> {
        let Some(latest_id_key_pair) = self.vault.latest_id_key_pair(now).await? else {
            anyhow::bail!(
                "No ID key pairs present in vault, cannot rotate to a new ID key pair. Use a journalist provisioning key pair to create a new seed ID key pair."
            )
        };

        let journalist_id = self.vault.journalist_id().await?;

        // Get or create candidate key pair
        tracing::debug!("Getting or creating candidate id key pair");
        let (candidate_id_key_pair, candidate_key_pair_added_at) =
            self.vault.get_or_create_candidate_id_key_pair(now).await?;

        // Check if the candidate key was successfully published in a previous attempt
        // to rotate the key
        if let Some(signed_id_pk_with_epoch) = self
            .api_client
            .get_journalist_id_pk_with_epoch(candidate_id_key_pair.public_key())
            .await?
        {
            tracing::info!(
                "Candidate key appears to have been rotated already, promoting vault key from candidate to published."
            );
            let epoch = signed_id_pk_with_epoch.epoch;

            // Promote to published
            self.vault
                .promote_candidate_id_key_pair_to_published(
                    candidate_id_key_pair,
                    candidate_key_pair_added_at,
                    signed_id_pk_with_epoch,
                    now,
                )
                .await?;

            tracing::info!("Rotated identity keys");
            return Ok(Some(epoch));
        }

        let candidate_id_pk = candidate_id_key_pair.public_key();

        //
        // The key did not successfully rotate on a previous iteration
        // we need to check if we need to reupload the form
        //

        tracing::debug!("Fetching journalist ID key pair forms");
        let current_queued_candidate_pks = self.api_client.get_journalist_id_pk_forms().await?;

        let maybe_our_candidate_pk = current_queued_candidate_pks
            .iter()
            .find(|f| f.journalist_id == journalist_id);

        // If there's no existing form in the API, or that form is expired
        // then (re)create the form and upload it
        let should_upload_form = maybe_our_candidate_pk.is_none()
            || maybe_our_candidate_pk
                .map(|form| form.form.not_valid_after() < now)
                .unwrap_or(false);

        // We haven't yet uploaded a candidate key to the queue, upload one now
        if should_upload_form {
            tracing::debug!("Uploading new form");
            let form_for_identity_api =
                RotateJournalistIdPublicKeyForm::new(candidate_id_pk, &latest_id_key_pair, now)?;

            // Form to submit the inner form to the api
            let form_for_api = RotateJournalistIdPublicKeyFormForm::new(
                form_for_identity_api,
                &latest_id_key_pair,
                now,
            )?;

            self.api_client
                .post_rotate_journalist_id_pk_form(form_for_api)
                .await?;
        }

        let mut maybe_signed_id_pk_with_epoch = None;

        // Poll the API to see if the ID key has been given an epoch...
        for _ in 0..JOURNALIST_ID_KEY_POLL_ITERATIONS {
            tokio::time::sleep(JOURNALIST_ID_KEY_POLL_SLEEP_DURATION).await;

            let polled_signed_id_pk_with_epoch = self
                .api_client
                .get_journalist_id_pk_with_epoch(candidate_id_pk)
                .await?;

            if polled_signed_id_pk_with_epoch.is_some() {
                maybe_signed_id_pk_with_epoch = polled_signed_id_pk_with_epoch;
                break;
            }
        }

        if let Some(signed_id_pk_with_epoch) = maybe_signed_id_pk_with_epoch {
            let epoch = signed_id_pk_with_epoch.epoch;

            // Promote to published
            self.vault
                .promote_candidate_id_key_pair_to_published(
                    candidate_id_key_pair,
                    candidate_key_pair_added_at,
                    signed_id_pk_with_epoch,
                    now,
                )
                .await?;

            tracing::info!("Rotated identity keys");
            return Ok(Some(epoch));
        }

        tracing::info!("No signed journalist identity public key after 60 seconds of polling");
        Ok(None)
    }

    /// Unconditionally rotate the journalist's messaging key pair.
    /// The caller is responsible for checking if rotation is needed.
    pub async fn rotate_msg_key(&self, now: DateTime<Utc>) -> Result<()> {
        // Get or create candidate key pair (vault/DB operation only)
        let candidate_msg_key_pair = self.vault.get_or_create_candidate_msg_key_pair(now).await?;

        // Get the latest ID key pair for signing the upload
        let Some(latest_id_key_pair) = self.vault.latest_id_key_pair(now).await? else {
            anyhow::bail!("No ID key pairs present in vault, cannot rotate messaging key pair")
        };

        // Upload to API
        let epoch = self
            .api_client
            .post_journalist_msg_pk(
                candidate_msg_key_pair.public_key(),
                &latest_id_key_pair,
                now,
            )
            .await?;

        // Promote candidate to published
        self.vault
            .promote_candidate_msg_key_pair(&candidate_msg_key_pair, epoch)
            .await?;

        tracing::info!("Rotated messaging keys");
        Ok(())
    }

    /// Check if identity key rotation is needed, and rotate if so.
    /// Returns true if rotation was performed, false if not needed.
    async fn check_and_rotate_id_key(&self, now: DateTime<Utc>) -> Result<bool> {
        if self.vault.latest_id_key_pair(now).await?.is_none() {
            anyhow::bail!(
                "No valid identity keys found in vault, cannot rotate any keys, this vault needs to be reseeded"
            );
        }

        let last_update = self
            .vault
            .last_published_id_key_pair_at()
            .await?
            .unwrap_or(DateTime::<Utc>::MIN_UTC); // shouldn't happen because of check above.
        let duration_since_last_update = (now - last_update).abs();

        if duration_since_last_update > JOURNALIST_ID_KEY_ROTATE_AFTER {
            self.rotate_id_key(now).await?;
            Ok(true)
        } else {
            let hours_elapsed = duration_since_last_update.num_hours();
            tracing::debug!(
                "Not refreshing identity keys since only {} hours have elapsed since the last rotation",
                hours_elapsed
            );
            Ok(false)
        }
    }

    /// Check if messaging key rotation is needed, and rotate if so.
    /// Returns true if rotation was performed, false if not needed.
    async fn check_and_rotate_msg_key(&self, now: DateTime<Utc>) -> Result<bool> {
        let duration_since_last_update = now.signed_duration_since(
            self.vault
                .last_published_msg_key_pair_at()
                .await?
                .unwrap_or(DateTime::<Utc>::MIN_UTC), // If we've never uploaded a key set to distant past
        );

        if duration_since_last_update > JOURNALIST_MSG_KEY_ROTATE_AFTER {
            self.rotate_msg_key(now).await?;
            Ok(true)
        } else {
            let hours_elapsed = duration_since_last_update.num_hours();
            tracing::debug!(
                "Not refreshing messaging keys since only {} hours have elapsed since the last rotation",
                hours_elapsed
            );
            Ok(false)
        }
    }

    /// Check if the journalist keys need to be rotated, if so, rotate them.
    /// Returns true if any keys were rotated, false if no rotation was needed.
    pub async fn check_and_rotate_keys(&self, now: DateTime<Utc>) -> Result<bool> {
        if self.vault.latest_id_key_pair(now).await?.is_none() {
            anyhow::bail!(
                "No valid identity keys found in vault, cannot rotate any keys, this vault needs to be reseeded"
            );
        }

        let mut did_rotate_some_keys = false;

        //
        // Identity key rotation
        //

        match self.check_and_rotate_id_key(now).await {
            Ok(true) => {
                did_rotate_some_keys = true;
            }
            Ok(false) => {}
            Err(e) => tracing::error!("Failed to refresh identity key: {:?}", e),
        }

        //
        // Messaging key rotation
        //

        match self.check_and_rotate_msg_key(now).await {
            Ok(true) => {
                did_rotate_some_keys = true;
            }
            Ok(false) => {}
            Err(e) => tracing::error!("Failed to refresh messaging key: {:?}", e),
        }

        Ok(did_rotate_some_keys)
    }

    /// Encrypt a message and enqueue it for sending to a user.
    /// Returns the queue length after enqueuing.
    pub async fn enqueue_j2u_message(
        &self,
        keys: &VerifiedKeysAndJournalistProfiles,
        user_pk: &UserPublicKey,
        message: &str,
        now: DateTime<Utc>,
    ) -> Result<i64> {
        let unencrypted_message = FixedSizeMessageText::new(message)?;

        let latest_journalist_msg_key_pair = self
            .vault
            .latest_msg_key_pair(now)
            .await?
            .ok_or_else(|| anyhow::anyhow!("No messaging keys in vault"))?;

        let encrypted_message = encrypt_real_message_from_journalist_to_user_via_covernode(
            &keys.keys,
            user_pk,
            &latest_journalist_msg_key_pair,
            &unencrypted_message,
        )?;

        let queue_length = self
            .vault
            .add_message_from_journalist_to_user_and_enqueue(
                user_pk,
                &unencrypted_message,
                encrypted_message,
                now,
            )
            .await?;

        Ok(queue_length)
    }

    /// Dequeue a j2u message for sending to a user, and send it to the API.
    /// If there are no messages to send, create and send a cover message.
    /// Returns the queue length after dequeuing.
    pub async fn dequeue_and_send_j2u_message(
        &self,
        keys: &OrganizationPublicKeyFamilyList,
        now: DateTime<Utc>,
    ) -> Result<i64> {
        let Some(id_key_pair) = self.vault.latest_id_key_pair(now).await? else {
            anyhow::bail!("No ID key pair found in vault");
        };

        if let Ok(Some(message)) = self.vault.head_queue_message().await {
            tracing::debug!("Found message in vault queue");

            self.api_client
                .post_journalist_msg(message.message, &id_key_pair, now)
                .await?;
            tracing::debug!("Posting message was successful, deleting message from queue");

            let queue_length = self.vault.delete_queue_message(message.id).await?;
            tracing::debug!("Successfully deleted message from queue");

            Ok(queue_length)
        } else {
            tracing::debug!("No message in vault queue, creating and sending cover message");

            let message = new_encrypted_cover_message_from_journalist_via_covernode(keys)?;

            self.api_client
                .post_journalist_msg(message, &id_key_pair, now)
                .await?;

            tracing::debug!("Posting message was successful");

            Ok(0)
        }
    }

    /// Return the list of users / sources known to this vault.
    pub async fn get_users(&self) -> Result<Vec<User>> {
        let users = self.vault.users().await?;
        Ok(users)
    }

    /// Process the vault setup bundle if one exists.
    pub async fn process_vault_setup_bundle(
        &self,
        now: DateTime<Utc>,
    ) -> Result<ProcessVaultSetupBundleResult> {
        let Some(vault_setup_bundle) = self.vault.get_vault_setup_bundle(now).await? else {
            let journalist_id = self.vault.journalist_id().await?;
            tracing::info!("Vault for {} has already been registered", journalist_id);
            return Ok(ProcessVaultSetupBundleResult::AlreadyRegistered);
        };

        let journalist_id = self.vault.journalist_id().await?;
        tracing::debug!("Found journalist vault setup bundle for {}", journalist_id);

        //
        // Set up journalist profile
        // Only done if this is the initial set up bundle, not a bundle to create a new key
        // after a journalist has been offline for too long.
        //

        if let Some(register_journalist_form) = vault_setup_bundle.register_journalist_form {
            tracing::debug!(
                "Uploading journalist registration form for {}",
                journalist_id
            );

            // This is the first time this vault has been seeded - we also need to upload the journalist to the API
            self.api_client
                .post_journalist_form(register_journalist_form)
                .await?;
        }

        //
        // Set up identity public key in API
        //

        tracing::debug!(
            "Uploading initial journalist public key to API for {}",
            journalist_id
        );

        let epoch = self
            .api_client
            .post_journalist_id_pk_form(vault_setup_bundle.pk_upload_form)
            .await?;

        //
        // Set up identity public key in vault
        //

        let vault_id_key_pairs = self
            .vault
            .id_key_pairs(now)
            .await?
            .find(|vault_id_key_pair| {
                vault_id_key_pair.public_key() == vault_setup_bundle.key_pair.public_key()
            });

        if vault_id_key_pairs.is_none() {
            tracing::debug!(
                "Inserting initial journalist public key into vault for {}",
                journalist_id
            );
            self.vault
                .insert_registered_id_key_pair(
                    vault_setup_bundle.provisioning_pk_id,
                    &vault_setup_bundle.key_pair,
                    now,
                    now,
                    epoch,
                )
                .await?;
        } else {
            tracing::warn!(
                "Journalist setup bundle is running but the key is already in the vault. This indicates a possible previous partial failure."
            );
        }

        // Attempt to set the max dead drop id to the current max
        // if we don't have any messaging keys.
        //
        // This isn't 100% required so if any part fails then just continue
        if let Ok(None) = self.vault.latest_msg_key_pair(now).await {
            if let Ok(recent_dead_drops_summary) = self
                .api_client
                .get_journalist_recent_dead_drop_summary()
                .await
            {
                if let Some(max_dead_drop_summary) = recent_dead_drops_summary
                    .iter()
                    .max_by_key(|summary| summary.id)
                {
                    if let Err(e) = self
                        .vault
                        .set_max_dead_drop_id(max_dead_drop_summary.id)
                        .await
                    {
                        tracing::error!("Failed to set max dead drop id {:?}", e);
                    }
                }
            }
        }

        //
        // Delete setup bundle from vault
        //

        tracing::debug!("Deleting vault setup bundle for {}", journalist_id);
        self.vault.delete_vault_setup_bundle().await?;

        tracing::debug!(
            "Generating initial messaging key pair for {}",
            journalist_id
        );

        // Generate initial messaging key pair
        self.rotate_msg_key(now).await?;

        tracing::debug!("Successfully setup vault for {}", journalist_id);

        // Return indicating that the seeding process ran
        Ok(ProcessVaultSetupBundleResult::SuccessfullyProcessedBundle)
    }
}
