mod backup_queries;
mod id_key_queries;
mod info_queries;
pub mod key_rows;
pub mod logging;
mod message_queries;
mod msg_key_queries;
pub mod provisioning_key_queries;
#[cfg(test)]
mod test_vault_clean_up;
mod user_queries;
mod vault_message;
pub mod vault_setup_bundle;

use anyhow::Context;
use key_rows::{
    AllVaultKeys, UntrustedCandidateJournalistIdKeyPairRow,
    UntrustedCandidateJournalistMessagingKeyPairRow, UntrustedJournalistProvisioningPublicKeyRow,
    UntrustedPublishedJournalistIdKeyPairRow, UntrustedPublishedJournalistMessagingKeyPairRow,
};
use std::{path::Path, time::Duration as StdDuration};
pub use vault_message::{J2UMessage, U2JMessage, VaultMessage};

use crate::info_queries::journalist_id;
use crate::logging::LoggingSession;
pub use backup_queries::BackupHistoryEntry;
use chrono::{DateTime, Utc};
use common::{
    api::{
        api_client::ApiClient,
        forms::{
            PostJournalistForm, PostJournalistIdPublicKeyForm, RotateJournalistIdPublicKeyFormForm,
        },
        models::{
            dead_drops::DeadDropId,
            journalist_id::JournalistIdentity,
            messages::{
                journalist_to_covernode_message::EncryptedJournalistToCoverNodeMessage,
                user_to_journalist_message_with_dead_drop_id::UserToJournalistMessageWithDeadDropId,
            },
        },
    },
    argon2_sqlcipher::Argon2SqlCipher,
    client::mailbox::mailbox_message::UserStatus,
    crypto::keys::{
        public_key::PublicKey,
        signing::{SignedPublicSigningKey, UnsignedSigningKeyPair},
    },
    epoch::Epoch,
    identity_api::{
        forms::post_rotate_journalist_id::RotateJournalistIdPublicKeyForm,
        models::UntrustedJournalistIdPublicKeyWithEpoch,
    },
    protocol::{
        constants::{
            JOURNALIST_ID_KEY_ROTATE_AFTER, JOURNALIST_MSG_KEY_ROTATE_AFTER,
            MESSAGE_VALID_FOR_DURATION,
        },
        keys::{
            generate_journalist_messaging_key_pair, verify_journalist_provisioning_pk,
            AnchorOrganizationPublicKey, AnchorOrganizationPublicKeys, JournalistIdKeyPair,
            JournalistMessagingKeyPair, JournalistProvisioningPublicKey, LatestKey,
            UnregisteredJournalistIdKeyPair, UserPublicKey,
        },
        roles::JournalistProvisioning,
    },
    FixedSizeMessageText,
};
use id_key_queries::{candidate_id_key_pair, insert_candidate_id_key_pair};
use logging::LogEntry;
use msg_key_queries::{
    candidate_msg_key_pair, insert_candidate_msg_key_pair,
    promote_candidate_msg_key_pair_to_published,
};
use sqlx::Acquire;
use sqlx::SqlitePool;

pub const VAULT_EXTENSION: &str = "vault";
pub const PASSWORD_EXTENSION: &str = "password";

pub type QueuedMessageId = i64;

pub struct EncryptedJournalistToCoverNodeMessageWithId {
    pub id: QueuedMessageId,
    pub message: EncryptedJournalistToCoverNodeMessage,
}

/// Some journalist vault functions can optionally replace an existing item. For example,
/// when adding a setup bundle you can either keep or replace the existing set up bundle.
///
/// This enum makes it clearer at the call site what is happening.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplacementStrategy {
    /// Keep an existing item
    Keep,
    /// Replace the existing item
    Replace,
}

pub struct User {
    pub user_pk: UserPublicKey,
    pub alias: Option<String>,
    pub description: Option<String>,
    pub status: UserStatus,
    pub marked_as_unread: bool,
}

#[derive(Clone)]
pub struct JournalistVault {
    pub pool: SqlitePool,
    trust_anchors: Vec<AnchorOrganizationPublicKey>,
}
impl JournalistVault {
    pub async fn create(
        path: impl AsRef<Path>,
        password: &str,
        journalist_id: &JournalistIdentity,
        journalist_provisioning_pks: &[JournalistProvisioningPublicKey],
        now: DateTime<Utc>,
        trust_anchors: Vec<AnchorOrganizationPublicKey>,
    ) -> anyhow::Result<Self> {
        let db = Argon2SqlCipher::new(path, password).await?;
        let pool = db.into_sqlite_pool();

        sqlx::migrate!().run(&pool).await?;

        let mut conn = pool.acquire().await?;

        info_queries::create_initial_info(&mut conn, journalist_id).await?;

        for journalist_provisioning_pk in journalist_provisioning_pks {
            provisioning_key_queries::insert_journalist_provisioning_pk(
                &mut conn,
                journalist_provisioning_pk,
                now,
            )
            .await?;
        }

        Ok(Self {
            pool,
            trust_anchors,
        })
    }

    pub async fn open(
        path: impl AsRef<Path>,
        password: &str,
        trust_anchors: Vec<AnchorOrganizationPublicKey>,
    ) -> anyhow::Result<Self> {
        if !path.as_ref().exists() {
            anyhow::bail!(
                "Path to journalist vault does not exist {}",
                path.as_ref().display()
            );
        }

        let db = Argon2SqlCipher::open_and_maybe_migrate_from_legacy(path, password).await?;
        let pool = db.into_sqlite_pool();

        sqlx::migrate!().run(&pool).await?;

        // Attempt to read out the journalist ID - if the encryption key is wrong this will fail
        let mut conn = pool.acquire().await?;
        let _ = info_queries::journalist_id(&mut conn).await?;

        Ok(Self {
            pool,
            trust_anchors,
        })
    }

    pub async fn check_password(&self, path: impl AsRef<Path>, password: &str) -> bool {
        Argon2SqlCipher::check_can_open_database_assuming_migrated(path, password).await
    }

    /// Changes the password of the vault.
    pub async fn change_password(&self, new_password: &str) -> anyhow::Result<()> {
        let mut conn = self.pool.acquire().await?;

        Argon2SqlCipher::rekey_database(&mut conn, new_password).await?;

        Ok(())
    }

    //
    // Logging
    //

    pub async fn add_session(&self, session_started_at: DateTime<Utc>) -> anyhow::Result<i64> {
        let mut conn = self.pool.acquire().await?;
        logging::insert_session(&mut conn, session_started_at).await
    }

    pub async fn add_log_entries(
        &self,
        session_id: i64,
        log_entry: &[LogEntry],
    ) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;

        logging::insert_log_entries(&mut tx, session_id, log_entry).await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn get_log_session_timeline(&self) -> anyhow::Result<Vec<LoggingSession>> {
        let mut conn = self.pool.acquire().await?;
        logging::get_session_timeline(&mut conn).await
    }

    pub async fn get_log_entries(
        &self,
        min_level: String,
        search_term: String,
        before: DateTime<Utc>,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<LogEntry>> {
        let mut conn = self.pool.acquire().await?;
        logging::select_log_entries(&mut conn, min_level, search_term, before, limit, offset).await
    }

    //
    // Info
    //

    pub async fn journalist_id(&self) -> anyhow::Result<JournalistIdentity> {
        let mut conn = self.pool.acquire().await?;
        info_queries::journalist_id(&mut conn).await
    }

    pub async fn last_published_id_key_pair_at(&self) -> anyhow::Result<Option<DateTime<Utc>>> {
        let mut conn = self.pool.acquire().await?;
        id_key_queries::last_published_id_key_pair_at(&mut conn).await
    }

    pub async fn last_published_msg_key_pair_at(&self) -> anyhow::Result<Option<DateTime<Utc>>> {
        let mut conn = self.pool.acquire().await?;
        msg_key_queries::last_published_msg_key_pair_at(&mut conn).await
    }

    pub async fn max_dead_drop_id(&self) -> anyhow::Result<DeadDropId> {
        let mut conn = self.pool.acquire().await?;
        info_queries::max_dead_drop_id(&mut conn).await
    }

    pub async fn all_vault_keys(&self, now: DateTime<Utc>) -> anyhow::Result<AllVaultKeys> {
        let mut conn = self.pool.begin().await?;

        let trust_anchors = self.trust_anchors()?;
        let org_pks_untrusted = trust_anchors.to_untrusted();

        let journalist_provisioning_pks = provisioning_key_queries::journalist_provisioning_pks(
            &mut conn,
            now,
            trust_anchors.clone(),
        )
        .await?
        .map(|row| UntrustedJournalistProvisioningPublicKeyRow {
            id: row.id,
            pk: row.pk.to_untrusted(),
        })
        .collect();

        let published_id_key_pairs =
            id_key_queries::published_id_key_pairs(&mut conn, now, trust_anchors.clone())
                .await?
                .map(|row| UntrustedPublishedJournalistIdKeyPairRow {
                    id: row.id,
                    key_pair: row.key_pair.to_untrusted(),
                    epoch: row.epoch,
                })
                .collect();

        let published_msg_key_pairs =
            msg_key_queries::published_msg_key_pairs(&mut conn, now, trust_anchors.clone())
                .await?
                .map(|row| UntrustedPublishedJournalistMessagingKeyPairRow {
                    id: row.id,
                    key_pair: row.key_pair.to_untrusted(),
                    epoch: row.epoch,
                })
                .collect();

        let candidate_id_key_pair =
            id_key_queries::candidate_id_key_pair(&mut conn)
                .await?
                .map(|row| UntrustedCandidateJournalistIdKeyPairRow {
                    id: row.id,
                    added_at: row.added_at,
                    key_pair: row.key_pair.to_untrusted(),
                });

        let candidate_msg_key_pair =
            msg_key_queries::candidate_msg_key_pair(&mut conn, now, trust_anchors.clone())
                .await?
                .map(|row| UntrustedCandidateJournalistMessagingKeyPairRow {
                    id: row.id,
                    added_at: row.added_at,
                    key_pair: row.key_pair.to_untrusted(),
                });

        Ok(AllVaultKeys {
            org_pks: org_pks_untrusted,
            journalist_provisioning_pks,
            candidate_msg_key_pair,
            candidate_id_key_pair,
            published_id_key_pairs,
            published_msg_key_pairs,
        })
    }

    //
    // Users
    //

    pub async fn users(&self) -> anyhow::Result<Vec<User>> {
        let mut conn = self.pool.acquire().await?;

        user_queries::users(&mut conn).await
    }

    pub async fn update_user_alias_and_description(
        &self,
        user_pk: &UserPublicKey,
        alias: &str,
        description: &str,
    ) -> anyhow::Result<()> {
        let mut conn = self.pool.acquire().await?;

        user_queries::update_user_alias_and_description(&mut conn, user_pk, alias, description)
            .await
    }

    //
    // Messages
    //

    pub async fn add_messages_from_user_to_journalist_and_update_max_dead_drop_id(
        &self,
        messages: &[UserToJournalistMessageWithDeadDropId],
        max_dead_drop_id: i32,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;

        for message in messages {
            // insert into users table if not already present
            user_queries::add_user(&mut tx, &message.u2j_message.reply_key, now).await?;

            message_queries::add_u2j_message(
                &mut tx,
                &message.u2j_message.reply_key,
                &message.u2j_message.message,
                now,
                message.dead_drop_id,
            )
            .await?;
        }

        info_queries::set_max_dead_drop_id(&mut tx, max_dead_drop_id).await?;

        tx.commit().await?;

        Ok(())
    }

    // TODO:
    // This should be the only option. The two partial versions of this are not
    // transaction safe.
    pub async fn add_message_from_journalist_to_user_and_enqueue(
        &self,
        user_pk: &UserPublicKey,
        // Maybe passing in both the encrypted and unencrypted is a bit weird but otherwise
        // we'd be passing in the public key hierarchy.
        unencrypted_message: &FixedSizeMessageText,
        encrypted_message: EncryptedJournalistToCoverNodeMessage,
        now: DateTime<Utc>,
    ) -> anyhow::Result<i64> {
        let mut tx = self.pool.begin().await?;

        // insert into users table if not already present
        user_queries::add_user(&mut tx, user_pk, now).await?;

        let queue_id = message_queries::enqueue_message(&mut tx, encrypted_message).await?;
        message_queries::add_j2u_message(
            &mut tx,
            user_pk,
            unencrypted_message,
            now,
            Some(queue_id),
        )
        .await?;

        let queue_length = message_queries::get_queue_length(&mut tx).await?;

        tx.commit().await?;

        Ok(queue_length)
    }

    /// Get the oldest message in a journalist's outbound queue
    pub async fn head_queue_message(
        &self,
    ) -> anyhow::Result<Option<EncryptedJournalistToCoverNodeMessageWithId>> {
        let mut conn = self.pool.acquire().await?;
        message_queries::peek_head_queue_message(&mut conn).await
    }

    /// Delete a message from the outbound queue and returns the new queue length.
    /// This should only be called after a message has successfully been sent to the Kinesis stream
    pub async fn delete_queue_message(&self, id: i64) -> anyhow::Result<i64> {
        let mut tx = self.pool.begin().await?;

        message_queries::delete_queue_message(&mut tx, id).await?;

        let new_queue_length = message_queries::get_queue_length(&mut tx).await?;

        tx.commit().await?;

        Ok(new_queue_length)
    }

    pub async fn mark_as_read(&self, user_pk: &UserPublicKey) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;
        message_queries::mark_as_read(&mut tx, user_pk).await?;
        user_queries::mark_as_read(&mut tx, user_pk).await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn mark_as_unread(&self, user_pk: &UserPublicKey) -> anyhow::Result<()> {
        let mut conn = self.pool.acquire().await?;
        user_queries::mark_as_unread(&mut conn, user_pk).await
    }

    pub async fn set_custom_expiry(
        &self,
        message: &VaultMessage,
        custom_expiry: Option<DateTime<Utc>>,
    ) -> anyhow::Result<()> {
        let mut conn = self.pool.acquire().await?;
        message_queries::set_custom_expiry(&mut conn, message, custom_expiry).await
    }

    pub async fn messages(&self) -> anyhow::Result<Vec<VaultMessage>> {
        let mut conn = self.pool.acquire().await?;

        message_queries::messages(&mut conn).await
    }

    pub async fn update_user_status(
        &self,
        user_pk: &UserPublicKey,
        status: UserStatus,
    ) -> anyhow::Result<()> {
        let mut conn = self.pool.acquire().await?;

        user_queries::update_user_status(&mut conn, user_pk, status).await
    }

    //
    // Keys
    //

    // TODO move toward using trust_anchors() and AnchorOrganizationPublicKeys
    pub fn org_pks(&self) -> anyhow::Result<Vec<AnchorOrganizationPublicKey>> {
        Ok(self.trust_anchors.clone())
    }

    pub fn trust_anchors(&self) -> anyhow::Result<AnchorOrganizationPublicKeys> {
        Ok(AnchorOrganizationPublicKeys::new(
            self.trust_anchors.clone(),
        ))
    }

    pub async fn provisioning_pks(
        &self,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Vec<JournalistProvisioningPublicKey>> {
        let mut conn = self.pool.acquire().await?;

        let trust_anchors = self.trust_anchors()?;
        let provisioning_keys =
            provisioning_key_queries::journalist_provisioning_pks(&mut conn, now, trust_anchors)
                .await?
                .map(|row| row.pk)
                .collect();

        Ok(provisioning_keys)
    }

    pub async fn add_provisioning_pk(
        &self,
        journalist_provisioning_pk: &JournalistProvisioningPublicKey,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let mut conn = self.pool.acquire().await?;

        provisioning_key_queries::insert_journalist_provisioning_pk(
            &mut conn,
            journalist_provisioning_pk,
            now,
        )
        .await
    }

    pub async fn id_key_pairs(
        &self,
        now: DateTime<Utc>,
    ) -> anyhow::Result<impl Iterator<Item = JournalistIdKeyPair>> {
        let mut conn = self.pool.acquire().await?;

        let trust_anchors = self.trust_anchors()?;
        let id_key_pairs = id_key_queries::published_id_key_pairs(&mut conn, now, trust_anchors)
            .await?
            .map(|row| row.key_pair);

        Ok(id_key_pairs)
    }

    pub async fn latest_id_key_pair(
        &self,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Option<JournalistIdKeyPair>> {
        let mut conn = self.pool.acquire().await?;

        let trust_anchors = self.trust_anchors()?;
        let latest_key_pair = id_key_queries::published_id_key_pairs(&mut conn, now, trust_anchors)
            .await?
            .map(|key_pair_row| key_pair_row.key_pair)
            .collect::<Vec<_>>()
            .into_latest_key();

        Ok(latest_key_pair)
    }

    pub async fn msg_key_pairs_for_decryption(
        &self,
        now: DateTime<Utc>,
    ) -> anyhow::Result<impl Iterator<Item = JournalistMessagingKeyPair>> {
        let mut conn = self.pool.acquire().await?;

        let trust_anchors = self.trust_anchors()?;
        let candidate_msg_key_pair =
            msg_key_queries::candidate_msg_key_pair(&mut conn, now, trust_anchors.clone())
                .await?
                .into_iter()
                .map(|row| row.key_pair);

        let published_msg_key_pairs =
            msg_key_queries::published_msg_key_pairs(&mut conn, now, trust_anchors)
                .await?
                .map(|iter| iter.key_pair);

        let combined_msg_key_pairs = candidate_msg_key_pair.chain(published_msg_key_pairs);

        Ok(combined_msg_key_pairs)
    }

    pub async fn latest_msg_key_pair(
        &self,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Option<JournalistMessagingKeyPair>> {
        let mut conn = self.pool.acquire().await?;

        let trust_anchors = self.trust_anchors()?;
        let latest_key_pair =
            msg_key_queries::published_msg_key_pairs(&mut conn, now, trust_anchors)
                .await?
                .map(|key_pair_row| key_pair_row.key_pair)
                .collect::<Vec<_>>()
                .into_latest_key();

        Ok(latest_key_pair)
    }

    pub async fn user_keys(&self) -> anyhow::Result<impl Iterator<Item = UserPublicKey> + '_> {
        let mut conn = self.pool.acquire().await?;

        user_queries::user_pks(&mut conn).await
    }

    /// Generates a new ID key pair and upload form, requires a journalist provisioning
    /// key pair. This is generally used when regular key rotation is impossible, such as
    /// when initially creating the vault (since there's no previous ID key pair) or when
    /// a journalist has not opened their app recently enough to allow rotation to happen
    /// and their ID key pairs have all expired.
    ///
    /// Since this requires a journalist provisioning key pair it is recommended that this
    /// is never done while connected to the internet or other untrusted network.
    pub async fn add_vault_setup_bundle(
        &self,
        journalist_provisioning_pk: &JournalistProvisioningPublicKey,
        journalist_id_key_pair: JournalistIdKeyPair,
        pk_upload_form: PostJournalistIdPublicKeyForm,
        register_journalist_form: Option<PostJournalistForm>,
        replacement_strategy: ReplacementStrategy,
    ) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;
        tx.begin().await?;

        let Some(provisioning_pk_id) =
            provisioning_key_queries::journalist_provisioning_pk_id_from_pk(
                &mut tx,
                journalist_provisioning_pk,
            )
            .await?
        else {
            anyhow::bail!(
                "Journalist provisioning key provided to add_vault_setup_bundle was not found in the vault"
            );
        };

        vault_setup_bundle::insert_vault_setup_bundle(
            &mut tx,
            provisioning_pk_id,
            &journalist_id_key_pair,
            pk_upload_form,
            register_journalist_form,
            replacement_strategy,
        )
        .await?;

        tx.commit().await?;

        Ok(())
    }

    /// Attempt to perform the initial vault set up.
    /// Returns `Ok(true)` if the set up occurred, and `Ok(false)` if there was no seed info to process.
    /// Any errors will return `Err(e)`.
    pub async fn process_vault_setup_bundle(
        &self,
        api_client: &ApiClient,
        now: DateTime<Utc>,
    ) -> anyhow::Result<bool> {
        let mut conn = self.pool.acquire().await?;

        let trust_anchors = self.trust_anchors()?;
        if let Some(vault_setup_bundle) =
            vault_setup_bundle::get_vault_setup_bundle(&mut *conn, now, trust_anchors).await?
        {
            let journalist_id = self.journalist_id().await?;

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
                api_client
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

            let epoch = api_client
                .post_journalist_id_pk_form(vault_setup_bundle.pk_upload_form)
                .await?;

            //
            // Set up identity public key in vault
            //

            let vault_id_key_pairs = self.id_key_pairs(now).await?.find(|vault_id_key_pair| {
                vault_id_key_pair.public_key() == vault_setup_bundle.key_pair.public_key()
            });

            if vault_id_key_pairs.is_none() {
                tracing::debug!(
                    "Inserting initial journalist public key into vault for {}",
                    journalist_id
                );
                id_key_queries::insert_registered_id_key_pair(
                    &mut conn,
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
            if let Ok(None) = self.latest_msg_key_pair(now).await {
                if let Ok(recent_dead_drops_summary) =
                    api_client.get_journalist_recent_dead_drop_summary().await
                {
                    if let Some(max_dead_drop_summary) = recent_dead_drops_summary
                        .iter()
                        .max_by_key(|summary| summary.id)
                    {
                        if let Err(e) =
                            info_queries::set_max_dead_drop_id(&mut conn, max_dead_drop_summary.id)
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
            vault_setup_bundle::delete_vault_setup_bundle(&mut *conn).await?;

            tracing::debug!(
                "Generating initial messaging key pair for {}",
                journalist_id
            );
            self.generate_msg_key_pair_and_upload_pk(api_client, now)
                .await?;

            tracing::debug!("Successfully setup vault for {}", journalist_id);

            // Return indicating that the seeding process ran
            Ok(true)
        } else {
            let journalist_id = journalist_id(&mut conn).await?;
            tracing::info!("Vault for {} has already been registered", journalist_id);

            // Return indicating that the seeding process did not run
            Ok(false)
        }
    }

    /// Takes an iterator of journalist provisioning keys and inserts any that aren't already in the vault
    /// after verifying them with trust anchors.
    pub async fn sync_journalist_provisioning_pks(
        &self,
        api_journalist_provisioning_pks: &Vec<&SignedPublicSigningKey<JournalistProvisioning>>,
        now: DateTime<Utc>,
    ) -> anyhow::Result<(), anyhow::Error> {
        let vault_journalist_provisioning_pks = self.provisioning_pks(now).await?;
        let journalist_provisioning_pks_to_insert: Vec<_> = api_journalist_provisioning_pks
            .iter()
            .filter(|key| !vault_journalist_provisioning_pks.contains(key))
            .collect();

        if journalist_provisioning_pks_to_insert.is_empty() {
            tracing::info!("No new provisioning keys from API to insert into vault");
            return Ok(());
        } else {
            tracing::info!(
                "Found {} new provisioning keys to add to vault",
                journalist_provisioning_pks_to_insert.len()
            )
        }

        let org_pks = self.org_pks()?;
        for journalist_provisioning_pk in journalist_provisioning_pks_to_insert {
            // find the trust anchor that has signed the provisioning key to insert
            let maybe_verified_journalist_provisioning_pk = org_pks.iter().find_map(|org_pk| {
                let org_pk = org_pk.to_non_anchor();
                verify_journalist_provisioning_pk(
                    &journalist_provisioning_pk.to_untrusted(),
                    &org_pk,
                    now,
                )
                .ok()
            });

            if let Some(journalist_provisioning_pk) = maybe_verified_journalist_provisioning_pk {
                tracing::info!(
                    "Found signing key for provisioning key. Inserting provisioning key."
                );
                self.add_provisioning_pk(&journalist_provisioning_pk, now)
                    .await?;
            } else {
                tracing::warn!(
                    "Could not find trust anchor for journalist provisioning public key {}",
                    journalist_provisioning_pk.public_key_hex()
                );
            };
        }

        Ok(())
    }

    /// Check if the journalist keys need to be rotated, if so, rotate them.
    pub async fn check_and_rotate_keys(
        &self,
        api_client: &ApiClient,
        now: DateTime<Utc>,
    ) -> anyhow::Result<bool> {
        if self.latest_id_key_pair(now).await?.is_none() {
            anyhow::bail!(
                "No valid identity keys found in vault, cannot rotate any keys, this vault needs to be reseeded"
            )
        };

        let mut did_rotate_some_keys = false;

        //
        // Identity key rotation
        //

        let last_update = self
            .last_published_id_key_pair_at()
            .await?
            .unwrap_or(DateTime::<Utc>::MIN_UTC);
        let duration_since_last_update = (now - last_update).abs();

        if duration_since_last_update > JOURNALIST_ID_KEY_ROTATE_AFTER {
            match self
                .generate_id_key_pair_and_rotate_pk(api_client, now)
                .await
            {
                Ok(_) => {
                    tracing::info!("Refreshed identity keys");
                    did_rotate_some_keys = true;
                }
                Err(e) => tracing::error!("Failed to refresh identity key: {:?}", e),
            }
        } else {
            let hours_elapsed = duration_since_last_update.num_hours();
            tracing::debug!(
                "Not refreshing identity keys since only {} hours have elapsed since the last rotation",
                hours_elapsed
            );
        }

        //
        // Messaging key rotation
        //

        let duration_since_last_update = now.signed_duration_since(
            self.last_published_msg_key_pair_at()
                .await?
                .unwrap_or(DateTime::<Utc>::MIN_UTC), // If we've never uploaded a key set to distant past
        );

        if duration_since_last_update > JOURNALIST_MSG_KEY_ROTATE_AFTER {
            match self
                .generate_msg_key_pair_and_upload_pk(api_client, now)
                .await
            {
                Ok(_) => {
                    tracing::info!("Refreshed messaging keys");
                    did_rotate_some_keys = true;
                }
                Err(e) => tracing::error!("Failed to refresh messaging key: {:?}", e),
            }
        } else {
            let hours_elapsed = duration_since_last_update.num_hours();
            tracing::debug!(
                "Not refreshing messaging keys since only {} hours have elapsed since the last rotation",
                hours_elapsed
            );
        }

        Ok(did_rotate_some_keys)
    }

    /// Generate a new ID key pair for this journalist and use the identity API to rotate to it
    ///
    /// Note that this function does *NOT* check if it's appropriate to rotate a key yet. That is,
    /// it will not check if sufficient time has passed since the last key rotation before attempting to
    /// upload a new key. This is primarily so that we can test that everything will still work in the cases
    /// where timings are not well behaved.
    ///
    /// Warning! This function will poll the API for up to 20 seconds to see if the key has been
    /// rotated. It should not be used where blocking flow for 20 seconds matters.
    ///
    /// Returns ok of some epoch if the key was successfully rotated this time. Returns ok
    /// of none if everything went ok but the identity-api didn't rotate it within 20 seconds.
    /// Returns an error if anything else went wrong, for example, if the API was unreachable.
    pub async fn generate_id_key_pair_and_rotate_pk(
        &self,
        api_client: &ApiClient,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Option<Epoch>> {
        let Some(latest_id_key_pair) = self.latest_id_key_pair(now).await? else {
            anyhow::bail!(
                "No ID key pairs present in vault, cannot rotate to a new ID key pair. Use a journalist provisioning key pair to create a new seed ID key pair."
            )
        };

        let journalist_id = self.journalist_id().await?;

        tracing::debug!("Checking if candidate id key pair exists");
        let (candidate_id_key_pair, candidate_key_pair_added_at) = {
            let mut conn = self.pool.acquire().await?;

            if let Some(candidate_id_key_pair_row) = candidate_id_key_pair(&mut conn).await? {
                tracing::debug!("Found candidate id key pair");

                let candidate_id_key_pair = candidate_id_key_pair_row.key_pair;
                let candidate_key_pair_added_at = candidate_id_key_pair_row.added_at;

                // Check if the candidate key was successfully published in a previous attempt
                // to rotate the key
                if let Some(signed_id_pk_with_epoch) = api_client
                    .get_journalist_id_pk_with_epoch(candidate_id_key_pair.public_key())
                    .await?
                {
                    tracing::info!(
                        "Candidate key appears to have been rotated already, promoting vault key from candidate to published. Candidate key {:?}",
                        candidate_id_key_pair.public_key_hex()
                    );
                    let epoch = signed_id_pk_with_epoch.epoch;

                    self.promote_candidate_id_key_pair_to_published(
                        candidate_id_key_pair,
                        candidate_key_pair_added_at,
                        signed_id_pk_with_epoch,
                        now,
                    )
                    .await?;

                    return Ok(Some(epoch));
                }

                (candidate_id_key_pair, candidate_key_pair_added_at)
            } else {
                tracing::debug!("Generating new candidate id key pair");
                let candidate_id_key_pair = UnsignedSigningKeyPair::generate();
                let candidate_key_pair_added_at = now;
                insert_candidate_id_key_pair(
                    &mut conn,
                    &candidate_id_key_pair,
                    candidate_key_pair_added_at,
                )
                .await?;
                (candidate_id_key_pair, candidate_key_pair_added_at)
            }
        };

        let candidate_id_pk = candidate_id_key_pair.public_key();

        //
        // The key did not successfully rotate on a previous iteration
        // we need to check if we need to reupload the form
        //

        tracing::debug!("Fetching journalist ID key pair forms");
        let current_queued_candidate_pks = api_client.get_journalist_id_pk_forms().await?;

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

            api_client
                .post_rotate_journalist_id_pk_form(form_for_api)
                .await?;
        }

        let mut maybe_signed_id_pk_with_epoch = None;

        // Poll for 60 seconds to see if the ID key has been given an epoch...
        for _ in 0..60 {
            tokio::time::sleep(StdDuration::from_secs(1)).await;

            let polled_signed_id_pk_with_epoch = api_client
                .get_journalist_id_pk_with_epoch(candidate_id_pk)
                .await?;

            if polled_signed_id_pk_with_epoch.is_some() {
                maybe_signed_id_pk_with_epoch = polled_signed_id_pk_with_epoch;
                break;
            }
        }

        if let Some(signed_id_pk_with_epoch) = maybe_signed_id_pk_with_epoch {
            let epoch = signed_id_pk_with_epoch.epoch;

            self.promote_candidate_id_key_pair_to_published(
                candidate_id_key_pair,
                candidate_key_pair_added_at,
                signed_id_pk_with_epoch,
                now,
            )
            .await?;

            return Ok(Some(epoch));
        }

        tracing::info!("No signed journalist identity public key after 20 seconds of polling");
        Ok(None)
    }

    async fn promote_candidate_id_key_pair_to_published(
        &self,
        candidate_id_key_pair: UnregisteredJournalistIdKeyPair,
        candidate_key_pair_created_at: DateTime<Utc>,
        signed_id_pk_with_epoch: UntrustedJournalistIdPublicKeyWithEpoch,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let signed_id_pk = signed_id_pk_with_epoch.key;
        let epoch = signed_id_pk_with_epoch.epoch;

        let mut tx = self.pool.begin().await?;

        let trust_anchors = self.trust_anchors()?;

        tracing::info!("Finding provisioning key");
        // HACK HACK HACK
        // Finding the correct signing key by attempting to verify all of them
        // we should probably just return it from the API
        let maybe_provisioning_pk_and_signed_id_pk =
            provisioning_key_queries::journalist_provisioning_pks(&mut tx, now, trust_anchors)
                .await?
                .find_map(|provisioning_key_pair_row| {
                    let provisioning_pk = &provisioning_key_pair_row.pk;
                    if let Ok(signed_id_pk) = signed_id_pk.to_trusted(provisioning_pk, now) {
                        Some((provisioning_key_pair_row.pk, signed_id_pk))
                    } else {
                        None
                    }
                });
        // End hack

        if let Some((provisioning_pk, signed_id_pk)) = maybe_provisioning_pk_and_signed_id_pk {
            tracing::info!("Found provisioning key, deleting existing candidate key pair");

            // Delete existing candidate ID key pair
            id_key_queries::delete_candidate_id_key_pair(&mut tx, &candidate_id_key_pair).await?;

            tracing::info!("Getting provisioning public key ID");

            let maybe_provisioning_pk_id =
                provisioning_key_queries::journalist_provisioning_pk_id_from_pk(
                    &mut tx,
                    &provisioning_pk,
                )
                .await?;

            let Some(provisioning_pk_id) = maybe_provisioning_pk_id else {
                anyhow::bail!("Provisioning key does not exist in journalist vault");
            };

            let id_key_pair =
                JournalistIdKeyPair::new(signed_id_pk, candidate_id_key_pair.secret_key);

            tracing::info!(
                "Inserting registered ID key pair: {}",
                id_key_pair.public_key_hex()
            );

            let published_at = now;
            id_key_queries::insert_registered_id_key_pair(
                &mut tx,
                provisioning_pk_id,
                &id_key_pair,
                candidate_key_pair_created_at,
                published_at,
                epoch,
            )
            .await?;
        } else {
            anyhow::bail!(
                "Failed to find parent provisioning public key while inserting new identity key pair"
            );
        }

        tx.commit().await?;

        Ok(())
    }

    pub async fn generate_msg_key_pair_and_upload_pk(
        &self,
        api_client: &ApiClient,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let Some(latest_id_key_pair) = self.latest_id_key_pair(now).await? else {
            anyhow::bail!(
                "No ID key pairs present in vault, cannot rotate to a new ID key pair. Use a journalist provisioning key pair to create a new seed ID key pair."
            )
        };

        let mut conn = self.pool.acquire().await?;

        let candidate_msg_key_pair = {
            let trust_anchors = self.trust_anchors()?;
            if let Some(candidate_msg_key_pair) =
                candidate_msg_key_pair(&mut conn, now, trust_anchors.clone()).await?
            {
                candidate_msg_key_pair.key_pair
            } else {
                let candidate_msg_key_pair =
                    generate_journalist_messaging_key_pair(&latest_id_key_pair, now);

                insert_candidate_msg_key_pair(
                    &mut conn,
                    latest_id_key_pair.public_key(),
                    &candidate_msg_key_pair,
                    now,
                    trust_anchors.clone(),
                )
                .await?;

                candidate_msg_key_pair
            }
        };

        let epoch = api_client
            .post_journalist_msg_pk(
                candidate_msg_key_pair.public_key(),
                &latest_id_key_pair,
                now,
            )
            .await?;

        promote_candidate_msg_key_pair_to_published(&mut conn, &candidate_msg_key_pair, epoch)
            .await?;

        Ok(())
    }

    /// - Delete expired id and msg key pairs
    /// - Delete expired provisioning public keys
    /// - Remove messages that are more than MESSAGE_VALID_FOR_DURATION old
    /// - Delete old logs
    pub async fn clean_up(&self, now: DateTime<Utc>) -> anyhow::Result<()> {
        let message_deletion_duration = MESSAGE_VALID_FOR_DURATION;

        let mut tx = self.pool.begin().await?;

        message_queries::delete_messages_before(&mut tx, now, message_deletion_duration)
            .await
            .context("delete old messages")?;

        // Delete expired keys
        msg_key_queries::delete_expired_msg_key_pairs(&mut tx, now)
            .await
            .context("delete expired msg key pairs")?;
        id_key_queries::delete_expired_id_key_pairs(&mut tx, now)
            .await
            .context("delete expired id key pairs")?;
        provisioning_key_queries::delete_expired_provisioning_pks(&mut tx, now)
            .await
            .context("delete expired provisioning pks")?;

        logging::delete_old_logs(&mut tx, now)
            .await
            .context("delete old logs")?;

        tx.commit().await?;

        let mut conn = self.pool.acquire().await?;
        sqlx::query!("VACUUM")
            .execute(&mut *conn)
            .await
            .context("vacuuming")?;

        Ok(())
    }

    //
    // Backups
    //

    pub async fn record_manual_backup(
        &self,
        timestamp: DateTime<Utc>,
        path: &str,
    ) -> anyhow::Result<()> {
        let mut conn = self.pool.acquire().await?;

        backup_queries::record_manual_backup(&mut conn, timestamp, path).await
    }

    pub async fn record_automated_backup(
        &self,
        timestamp: DateTime<Utc>,
        recovery_contact_journalist_ids: Vec<JournalistIdentity>,
    ) -> anyhow::Result<()> {
        let mut conn = self.pool.acquire().await?;

        backup_queries::record_automated_backup(
            &mut conn,
            timestamp,
            recovery_contact_journalist_ids,
        )
        .await
    }

    pub async fn get_count_of_keys_created_since_last_backup(&self) -> anyhow::Result<i64> {
        let mut conn = self.pool.acquire().await?;

        backup_queries::get_count_of_keys_created_since_last_backup(&mut conn).await
    }

    pub async fn get_backup_contacts(&self) -> anyhow::Result<Vec<JournalistIdentity>> {
        let mut conn = self.pool.acquire().await?;

        backup_queries::get_backup_contacts(&mut conn).await
    }

    pub async fn set_backup_contacts(
        &self,
        contacts: Vec<JournalistIdentity>,
    ) -> anyhow::Result<()> {
        let mut conn = self.pool.acquire().await?;

        backup_queries::set_backup_contacts(&mut conn, contacts).await
    }

    pub async fn get_backup_history(&self) -> anyhow::Result<Vec<BackupHistoryEntry>> {
        let mut conn = self.pool.acquire().await?;

        backup_queries::get_backup_history(&mut conn).await
    }

    pub async fn remove_invalid_backup_contacts(
        &self,
        journalist_identities_from_api: Vec<&JournalistIdentity>,
    ) -> anyhow::Result<u64> {
        let mut conn = self.pool.acquire().await?;

        backup_queries::remove_invalid_backup_contacts(&mut conn, journalist_identities_from_api)
            .await
    }
}
