use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use common::{
    api::{api_client::ApiClient, forms::PostCoverNodeMessagingPublicKeyForm},
    crypto::keys::{public_key::PublicKey, signing::SignedSigningKeyPair},
    identity_api::{
        client::IdentityApiClient, forms::post_rotate_covernode_id::RotateCoverNodeIdPublicKeyForm,
    },
    protocol::keys::{
        CoverDropPublicKeyHierarchy, CoverNodeIdKeyPair, CoverNodeIdKeyPairWithEpoch,
        CoverNodeMessagingKeyPairWithEpoch, LatestKey,
    },
    task::Task,
    time,
};
use covernode_database::{
    UntrustedCandidateCoverNodeIdKeyPairWithCreatedAt,
    UntrustedCandidateCoverNodeMessagingKeyPairWithCreatedAt,
};

use crate::key_state::KeyState;

pub struct PublishedKeysTask {
    interval: Duration,
    key_state: KeyState,
    api_client: ApiClient,
    identity_api_client: IdentityApiClient,
}

#[async_trait]
impl Task for PublishedKeysTask {
    fn name(&self) -> &'static str {
        "publish_keys"
    }

    async fn run(&self) -> anyhow::Result<()> {
        tracing::debug!("Running publish keys task");
        let now = time::now();

        let mut new_id_key_pair_and_epoch = None;
        let mut new_msg_key_pair_and_epoch = None;

        // We have an RwLock around the key state so we should do all the operations that require
        // a read lock first - and only if we have modifications to the key state should we take a
        // write lock.
        {
            let key_state = self.key_state.read().await;

            let keys = self
                .api_client
                .get_public_keys()
                .await?
                .into_trusted(key_state.anchor_org_pks(), now)
                .keys;

            let published_covernode_id_key_pairs = key_state.published_covernode_id_key_pairs();
            let latest_id_key_pair = published_covernode_id_key_pairs.latest_key_required()?;

            // Ok, we've ran initial set up, now we can check for candidate keys
            let maybe_candidate_id_key_pair_with_created_at =
                key_state.get_candidate_id_key_pair().await?;

            let maybe_candidate_msg_key_pair_with_created_at =
                key_state.get_candidate_msg_key_pair().await?;

            if let Some(candidate_id_key_pair_with_created_at) =
                maybe_candidate_id_key_pair_with_created_at
            {
                let publish_id_key_pair_attempt = self
                    .process_candidate_id_key_pair(
                        candidate_id_key_pair_with_created_at,
                        &keys,
                        &latest_id_key_pair.key_pair,
                        now,
                    )
                    .await;

                match publish_id_key_pair_attempt {
                    Ok(id_key_pair_and_epoch) => {
                        new_id_key_pair_and_epoch = Some(id_key_pair_and_epoch)
                    }
                    Err(e) => tracing::error!("Failed to publish ID public key {}", e),
                }
            };

            if let Some(candidate_msg_key_pair_with_created_at) =
                maybe_candidate_msg_key_pair_with_created_at
            {
                let publish_msg_key_pair_attempt = self
                    .process_msg_key_pair(
                        candidate_msg_key_pair_with_created_at,
                        published_covernode_id_key_pairs,
                        latest_id_key_pair,
                        now,
                    )
                    .await;

                match publish_msg_key_pair_attempt {
                    Ok(msg_key_pair_and_epoch) => {
                        new_msg_key_pair_and_epoch = Some(msg_key_pair_and_epoch)
                    }
                    Err(e) => tracing::error!("Failed to publish messaging public key {}", e),
                }
            }
        }

        // Get write lock to update the values in our local CoverNode keys database
        let has_published_keys =
            new_id_key_pair_and_epoch.is_some() || new_msg_key_pair_and_epoch.is_some();

        if has_published_keys {
            let mut key_state = self.key_state.write().await;

            if let Some(new_id_key_pair_with_epoch) = new_id_key_pair_and_epoch {
                let update_id_key_pair_epoch_attempt = key_state
                    .add_epoch_to_covernode_id_key_pair(
                        new_id_key_pair_with_epoch.key_pair,
                        new_id_key_pair_with_epoch.epoch,
                        new_id_key_pair_with_epoch.created_at,
                    )
                    .await;

                if let Err(e) = update_id_key_pair_epoch_attempt {
                    tracing::error!("Failed to add epoch to ID key pair in database: {:?}", e);
                }
            }

            if let Some(new_msg_key_pair_with_epoch) = new_msg_key_pair_and_epoch {
                let update_msg_key_pair_epoch_attempt = key_state
                    .add_epoch_to_covernode_msg_key_pair(
                        new_msg_key_pair_with_epoch.key_pair,
                        new_msg_key_pair_with_epoch.epoch,
                        new_msg_key_pair_with_epoch.created_at,
                    )
                    .await;

                if let Err(e) = update_msg_key_pair_epoch_attempt {
                    tracing::error!(
                        "Failed to add epoch to messaging key pair in database: {:?}",
                        e
                    );
                }
            }
        }

        Ok(())
    }

    fn interval(&self) -> Duration {
        self.interval
    }
}

impl PublishedKeysTask {
    pub const ALERT_AFTER_CANDIDATE_UPLOAD_FAILURE_MINUTES: i64 = 60;
    pub fn new(
        interval: Duration,
        key_state: KeyState,
        api_client: ApiClient,
        identity_api_client: IdentityApiClient,
    ) -> Self {
        Self {
            interval,
            key_state,
            api_client,
            identity_api_client,
        }
    }

    async fn process_candidate_id_key_pair(
        &self,
        candidate_id_key_pair_with_created_at: UntrustedCandidateCoverNodeIdKeyPairWithCreatedAt,
        keys: &CoverDropPublicKeyHierarchy,
        latest_id_key_pair: &CoverNodeIdKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<CoverNodeIdKeyPairWithEpoch> {
        let UntrustedCandidateCoverNodeIdKeyPairWithCreatedAt {
            key_pair: candidate_id_key_pair,
            created_at: candidate_created_at,
        } = candidate_id_key_pair_with_created_at;

        if now - candidate_created_at
            > chrono::Duration::minutes(
                PublishedKeysTask::ALERT_AFTER_CANDIDATE_UPLOAD_FAILURE_MINUTES,
            )
        {
            tracing::warn!("Could not publish candidate identity public key after 60 minutes");

            metrics::counter!("KeyUploadFailure").increment(1);
        }

        tracing::debug!("Found candidate ID key pair, attempting to publish to API");

        let candidate_id_public_key = candidate_id_key_pair.public_key.to_trusted();

        let form =
            RotateCoverNodeIdPublicKeyForm::new(&candidate_id_public_key, latest_id_key_pair, now)?;

        let signed_id_pk_with_epoch = self
            .identity_api_client
            .post_rotate_covernode_id_key(form)
            .await?;

        // It's possible that there was a provisioning PK rotation between the key hierarchy being pulled at the top of this function
        // and now, at which point this CoverNode will NOT have access to the CoverNode provisioning PK, and will
        // be unable to verify the CoverNode ID public key that has just been returned by the identity API. If that happens
        // we throw and error and the system will attempt to reupload the same candidate key, which will return the previous
        // signed CoverNode ID public key, and hopefully this CoverNode will have the new provisioning key by then, so we can
        // continue and store the epoch and signed key pair in the CoverNode key database.
        let covernode_provisioning_pk = keys
            .latest_covernode_provisioning_pk()
            .ok_or_else(|| anyhow::anyhow!("No covernode provisioning public key in API"))?;

        // Confirm we trust the key that has just been given to us
        let signed_id_pk = signed_id_pk_with_epoch
            .key
            .to_trusted(covernode_provisioning_pk, now)
            .inspect_err(|e| {
                tracing::warn!(
                    "identity service returned covernode id key pair which we cannot validate: {}",
                    e
                );
            })?;

        // Create a new key pair, containing the signed public key and the candidate secret key.
        let new_id_key_pair =
            SignedSigningKeyPair::new(signed_id_pk, candidate_id_key_pair.secret_key);

        tracing::debug!(
            "Saving new id key pair ({}) to database",
            &new_id_key_pair.public_key_hex()[..8]
        );

        Ok(CoverNodeIdKeyPairWithEpoch::new(
            new_id_key_pair,
            signed_id_pk_with_epoch.epoch,
            candidate_created_at,
        ))
    }

    async fn process_msg_key_pair(
        &self,
        candidate_msg_key_pair_with_created_at: UntrustedCandidateCoverNodeMessagingKeyPairWithCreatedAt,
        id_key_pairs: &[CoverNodeIdKeyPairWithEpoch],
        latest_id_key_pair: &CoverNodeIdKeyPairWithEpoch,
        now: DateTime<Utc>,
    ) -> anyhow::Result<CoverNodeMessagingKeyPairWithEpoch> {
        let UntrustedCandidateCoverNodeMessagingKeyPairWithCreatedAt {
            key_pair: candidate_msg_key_pair,
            created_at: candidate_created_at,
        } = candidate_msg_key_pair_with_created_at;

        // TODO attempt to validate the candidate key in the form
        // if we cannot validate it against our set of ID keys then
        // we should delete it.
        if now - candidate_created_at
            > chrono::Duration::minutes(
                PublishedKeysTask::ALERT_AFTER_CANDIDATE_UPLOAD_FAILURE_MINUTES,
            )
        {
            tracing::warn!("Could not publish candidate messaging public key after 60 minutes");

            metrics::counter!("KeyUploadFailure").increment(1);
        }

        tracing::debug!("Found candidate messaging key pair, attempting to publish to API");

        // We need to validate the candidate messaging key pair using all available id public keys
        // this is because the if key could have rotated since the messaging candidate key was created
        // we reverse the iterator as an optimization because the last id key is most likely the parent
        let signed_msg_key_pair = candidate_msg_key_pair
            .to_trusted_from_candidate_parents(id_key_pairs.iter().rev(), now);

        if let Ok(signed_msg_key_pair) = signed_msg_key_pair {
            let form = PostCoverNodeMessagingPublicKeyForm::new(
                candidate_msg_key_pair.public_key,
                &latest_id_key_pair.key_pair,
                now,
            )?;

            let new_epoch = self.api_client.post_covernode_msg_pk_form(form).await?;
            Ok(CoverNodeMessagingKeyPairWithEpoch::new(
                signed_msg_key_pair,
                new_epoch,
                candidate_created_at,
            ))
        } else {
            // This is bad. Somehow we've managed to generate a key that cannot be validated with any
            // of our own identity keys.
            tracing::warn!(
                "Could not validate the candidate messaging key pair using any of the available id public keys"
            );

            metrics::counter!("CandidateMsgKeyValidationFailure").increment(1);

            anyhow::bail!("Could not validate the candidate messaging key pair using any of the available id public keys")
        }
    }
}
