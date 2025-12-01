use crate::key_state::KeyState;
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use common::{
    protocol::{
        constants::{
            COVERNODE_ID_KEY_ROTATE_AFTER_SECONDS, COVERNODE_MSG_KEY_ROTATE_AFTER_SECONDS,
        },
        keys::{
            generate_covernode_messaging_key_pair, generate_unregistered_covernode_id_key_pair,
            CoverNodeIdKeyPairWithEpoch, CoverNodeMessagingKeyPair,
            CoverNodeMessagingKeyPairWithEpoch, LatestKey, UnregisteredCoverNodeIdKeyPair,
        },
    },
    task::Task,
    time,
};

pub struct CreateKeysTask {
    interval: Duration,
    key_state: KeyState,
}

impl CreateKeysTask {
    pub fn new(interval: Duration, key_state: KeyState) -> Self {
        Self {
            interval,
            key_state,
        }
    }

    fn maybe_create_candidate_id_key_pair(
        published_id_key_pairs: &[CoverNodeIdKeyPairWithEpoch],
        now: DateTime<Utc>,
    ) -> anyhow::Result<Option<UnregisteredCoverNodeIdKeyPair>> {
        // We have at least one valid id key pair, so we need to check if it is old
        // enough to require a rotation
        if let Some(latest_id_key_pair) = published_id_key_pairs.latest_key() {
            let key_pair_created_at = latest_id_key_pair.created_at;

            let key_pair_rotate_at =
                key_pair_created_at + Duration::seconds(COVERNODE_ID_KEY_ROTATE_AFTER_SECONDS);

            tracing::info!(
                "Checking latest identity key rotation time ({}) against now ({})",
                key_pair_rotate_at,
                now
            );

            if now > key_pair_rotate_at {
                tracing::info!("Past due rotating our identity keys");
                return Ok(Some(generate_unregistered_covernode_id_key_pair()));
            }
        } else {
            // We have no valid id keys, rotate candidate key pair immediately
            // It's responsibility of the publish keys task to alert that this
            // candidate key pair cannot be signed
            tracing::warn!("No valid identity keys, creating a new one which won't be publishable without manual intervention");
            return Ok(Some(generate_unregistered_covernode_id_key_pair()));
        }

        Ok(None)
    }

    fn maybe_create_candidate_msg_key_pair(
        published_id_key_pairs: &[CoverNodeIdKeyPairWithEpoch],
        published_msg_key_pairs: &[CoverNodeMessagingKeyPairWithEpoch],
        now: DateTime<Utc>,
    ) -> anyhow::Result<Option<CoverNodeMessagingKeyPair>> {
        let latest_id_key_pair = published_id_key_pairs
            .latest_key_required()
            .map(|k| &k.key_pair)?;

        // We have at least one valid msg key pair, so we need to check if it is old
        // enough to require a rotation
        if let Some(latest_msg_key_pair_with_epoch) = published_msg_key_pairs.latest_key() {
            let key_pair_created_at = latest_msg_key_pair_with_epoch.created_at;
            let key_pair_rotate_at =
                key_pair_created_at + Duration::seconds(COVERNODE_MSG_KEY_ROTATE_AFTER_SECONDS);

            tracing::info!(
                "Checking latest messaging key rotation time ({}) against now ({})",
                key_pair_rotate_at,
                now
            );

            if now > key_pair_rotate_at {
                tracing::info!("Past due rotating a messaging key");
                return Ok(Some(generate_covernode_messaging_key_pair(
                    latest_id_key_pair,
                    now,
                )));
            }
        } else {
            tracing::info!("No valid messaging keys, rotating immediately");
            return Ok(Some(generate_covernode_messaging_key_pair(
                latest_id_key_pair,
                now,
            )));
        }

        Ok(None)
    }
}

#[async_trait]
impl Task for CreateKeysTask {
    fn name(&self) -> &'static str {
        "create_keys"
    }

    async fn run(&self) -> anyhow::Result<()> {
        let now = time::now();

        let (maybe_new_id_key_pair, maybe_new_msg_key_pair) = {
            let key_state = self.key_state.read().await;

            let setup_bundle = key_state.get_setup_bundle().await?;
            if setup_bundle.is_some() {
                tracing::info!(
                    "Skipping key creation because we have not yet processed setup bundle"
                );
                return Ok(());
            }

            let mut maybe_new_id_key_pair = None;
            let mut maybe_new_msg_key_pair = None;

            let maybe_candidate_id_key_pair = key_state.get_candidate_id_key_pair().await?;
            let maybe_candidate_msg_key_pair = key_state.get_candidate_msg_key_pair().await?;

            if maybe_candidate_id_key_pair.is_none() {
                tracing::info!(
                    "No candidate identity key found, checking if we should create one..."
                );
                maybe_new_id_key_pair = Self::maybe_create_candidate_id_key_pair(
                    key_state.published_covernode_id_key_pairs(),
                    now,
                )?;
            }

            if maybe_candidate_msg_key_pair.is_none() {
                tracing::info!(
                    "No candidate messaging key found, checking if we should create one..."
                );
                maybe_new_msg_key_pair = Self::maybe_create_candidate_msg_key_pair(
                    key_state.published_covernode_id_key_pairs(),
                    key_state.published_covernode_msg_key_pairs(),
                    now,
                )?;
            }

            (maybe_new_id_key_pair, maybe_new_msg_key_pair)
        };

        {
            let mut key_state = self.key_state.write().await;

            if let Some(new_id_key_pair) = maybe_new_id_key_pair {
                key_state
                    .insert_candidate_id_key_pair(new_id_key_pair, now)
                    .await?;
            }

            if let Some(new_msg_key_pair) = maybe_new_msg_key_pair {
                key_state
                    .insert_candidate_msg_key_pair(new_msg_key_pair, now)
                    .await?;
            }
        }

        Ok(())
    }

    fn interval(&self) -> Duration {
        self.interval
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::services::tasks::CreateKeysTask;
    use chrono::{DateTime, Utc};
    use common::{
        epoch::Epoch,
        protocol::{
            constants::{
                COVERNODE_ID_KEY_ROTATE_AFTER_SECONDS, COVERNODE_ID_KEY_VALID_DURATION_SECONDS,
                COVERNODE_MSG_KEY_ROTATE_AFTER_SECONDS,
                COVERNODE_PROVISIONING_KEY_VALID_DURATION_SECONDS,
            },
            keys::{
                generate_covernode_id_key_pair, generate_covernode_messaging_key_pair,
                generate_covernode_provisioning_key_pair, generate_organization_key_pair,
                CoverNodeIdKeyPairWithEpoch, CoverNodeMessagingKeyPair,
                CoverNodeMessagingKeyPairWithEpoch, CoverNodeProvisioningKeyPair,
                UnregisteredCoverNodeIdKeyPair,
            },
        },
        time::now,
    };

    fn create_test_provisioning_key(created_at: DateTime<Utc>) -> CoverNodeProvisioningKeyPair {
        let org_key = generate_organization_key_pair(created_at);
        generate_covernode_provisioning_key_pair(&org_key, created_at)
    }

    fn create_test_covernode_id_key_pair(
        created_at: DateTime<Utc>,
        provisioning_key: CoverNodeProvisioningKeyPair,
    ) -> CoverNodeIdKeyPairWithEpoch {
        let id_key_pair = generate_covernode_id_key_pair(&provisioning_key, created_at);
        CoverNodeIdKeyPairWithEpoch::new(id_key_pair, Epoch(1), created_at)
    }

    fn create_test_covernode_msg_key_pair(
        created_at: DateTime<Utc>,
        covernode_id_key_pair: CoverNodeIdKeyPairWithEpoch,
    ) -> CoverNodeMessagingKeyPairWithEpoch {
        let msg_key_pair =
            generate_covernode_messaging_key_pair(&covernode_id_key_pair.key_pair, created_at);
        CoverNodeMessagingKeyPairWithEpoch::new(msg_key_pair, Epoch(1), created_at)
    }

    fn create_test_candidate_covernode_id_key_pair(
        pairs: &[CoverNodeIdKeyPairWithEpoch],
        now: DateTime<Utc>,
    ) -> Option<UnregisteredCoverNodeIdKeyPair> {
        CreateKeysTask::maybe_create_candidate_id_key_pair(pairs, now).unwrap()
    }

    fn create_test_candidate_covernode_msg_key_pair(
        covernode_id_key_pairs: &[CoverNodeIdKeyPairWithEpoch],
        covernode_msg_key_pairs: &[CoverNodeMessagingKeyPairWithEpoch],
    ) -> Option<CoverNodeMessagingKeyPair> {
        CreateKeysTask::maybe_create_candidate_msg_key_pair(
            covernode_id_key_pairs,
            covernode_msg_key_pairs,
            now(),
        )
        .unwrap()
    }

    /// Scenario 1:
    /// Id key is less than COVERNODE_ID_KEY_ROTATE_AFTER_SECONDS old → no rotation should occur.
    #[tokio::test]
    async fn id_key_less_than_rotation_period_no_rotation() {
        let provisioning_key = create_test_provisioning_key(now());
        let id_key_pair = create_test_covernode_id_key_pair(now(), provisioning_key);
        let published_identity_key_pairs = vec![id_key_pair];

        let candidate_id_key_pair =
            create_test_candidate_covernode_id_key_pair(&published_identity_key_pairs, now());
        assert!(
            candidate_id_key_pair.is_none(),
            "Expected no key rotation for recent key"
        );
    }

    /// Scenario 2:
    /// Id key is older than COVERNODE_ID_KEY_ROTATE_AFTER_SECONDS → rotation required.
    #[tokio::test]
    async fn id_key_older_than_rotation_period_requires_rotation() {
        let old_time = now()
            - Duration::from_secs(
                (COVERNODE_ID_KEY_ROTATE_AFTER_SECONDS + 1)
                    .try_into()
                    .unwrap(),
            );
        let provisioning_key = create_test_provisioning_key(old_time);
        let keypair = create_test_covernode_id_key_pair(old_time, provisioning_key);
        let published = vec![keypair];

        let candidate = create_test_candidate_covernode_id_key_pair(&published, now());
        assert!(candidate.is_some(), "Expected key rotation for old key");
    }

    /// Scenario 3:
    /// Parent provisioning key close to expiry so ID key has a truncated expiry date,
    /// but is still within COVERNODE_ID_KEY_ROTATE_AFTER_SECONDS  - no rotation should occur.
    ///
    /// | ---- org_key ------------------------------------------|
    /// | ---- provisioning_key -----------------|
    ///                            |-- id_key ---| <- Truncated expiry date but still recent enough to not require rotation
    ///                                         ^ now() is target time we try and rotate the ID key
    ///
    #[tokio::test]
    async fn id_key_has_truncated_expiry_but_age_less_than_rotation_period_no_rotation() {
        let covernode_provisioning_key_valid_duration_seconds: u64 =
            COVERNODE_PROVISIONING_KEY_VALID_DURATION_SECONDS
                .try_into()
                .unwrap();

        let near_expiry = now()
            - Duration::from_secs(covernode_provisioning_key_valid_duration_seconds - 3600 * 7);

        // By generating the keys this way, we ensure that the provisioning key expiry is 7 hours from now()
        let provisioning_key = create_test_provisioning_key(near_expiry);

        // This ID key is recent enough to avoid rotation ie only 7 hours old
        let id_recent_date = now() - Duration::from_secs(3600 * 7);
        // As the parent provisioning key is near expiry, this means that when a child ID key is generated, it will have a truncated expiry date too
        let id_key = generate_covernode_id_key_pair(&provisioning_key, id_recent_date);

        // Sanity check - ensure the ID key expiry is the same as the parent provisioning key
        assert_eq!(
            id_key.public_key().not_valid_after,
            provisioning_key.public_key().not_valid_after
        );

        let keypair = CoverNodeIdKeyPairWithEpoch::new(id_key, Epoch(1), id_recent_date);
        let published = vec![keypair];

        let candidate = create_test_candidate_covernode_id_key_pair(&published, now());
        assert!(
            candidate.is_none(),
            "Expected no rotation when provisioning key near expiry but ID key is fresh"
        );
    }
    /// Scenario 1:
    /// Msg Key is less than COVERNODE_MSG_KEY_ROTATE_AFTER_SECONDS  → no rotation should occur.
    #[tokio::test]
    async fn msg_key_is_recently_created_no_rotation() {
        let provisioning_key = create_test_provisioning_key(now());
        let covernode_id_key_pair = create_test_covernode_id_key_pair(now(), provisioning_key);
        let published_covernode_id_key_pair = vec![covernode_id_key_pair.clone()];

        let covernode_msg_key_pair =
            create_test_covernode_msg_key_pair(now(), covernode_id_key_pair.clone());
        let published_covernode_msg_key_pair = vec![covernode_msg_key_pair.clone()];

        let candidate_msg_public_key = create_test_candidate_covernode_msg_key_pair(
            &published_covernode_id_key_pair,
            &published_covernode_msg_key_pair,
        );
        assert!(
            candidate_msg_public_key.is_none(),
            "Expected no key rotation for recent key"
        );
    }

    /// Scenario 2:
    /// Msg Key is older than COVERNODE_MSG_KEY_ROTATE_AFTER_SECONDS  → no rotation should occur.
    #[tokio::test]
    async fn msg_key_older_than_rotation_period_requires_rotation() {
        let old_time = now()
            - Duration::from_secs(
                (COVERNODE_MSG_KEY_ROTATE_AFTER_SECONDS + 1)
                    .try_into()
                    .unwrap(),
            );
        let provisioning_key = create_test_provisioning_key(old_time);
        let covernode_id_key_pair = create_test_covernode_id_key_pair(old_time, provisioning_key);
        let published_covernode_id_key_pair = vec![covernode_id_key_pair.clone()];

        let covernode_msg_key_pair =
            create_test_covernode_msg_key_pair(old_time, covernode_id_key_pair.clone());
        let published_covernode_msg_key_pair = vec![covernode_msg_key_pair.clone()];

        let candidate_msg_public_key = create_test_candidate_covernode_msg_key_pair(
            &published_covernode_id_key_pair,
            &published_covernode_msg_key_pair,
        );
        assert!(
            candidate_msg_public_key.is_some(),
            "Expected key rotation for old key"
        );
    }

    /// Scenario 3:
    /// Msg key has truncated expiry but is less than COVERNODE_MSG_KEY_ROTATE_AFTER_SECONDS - no rotation should occur.
    ///
    /// | ---- org_key ----------------------------------------------------------|
    /// | ---- provisioning_key -----------------------------|
    ///                            |-- id_key ---------------| <-- truncated id key lifetime
    ///                                       |-- msg_key ---| <-- truncated msg key lifetime but still recent enough to not require rotation
    ///                                                   ^ now() is target time we try and rotate the msg key
    ///
    #[tokio::test]
    async fn msg_key_near_expiry_but_still_less_that_rotation_period_no_rotation() {
        let covernode_provisioning_key_valid_duration_seconds: u64 =
            COVERNODE_PROVISIONING_KEY_VALID_DURATION_SECONDS
                .try_into()
                .unwrap();
        let provisioning_near_expiry =
            now() - Duration::from_secs(covernode_provisioning_key_valid_duration_seconds - 60 * 5);

        let provisioning_key = create_test_provisioning_key(provisioning_near_expiry);

        let covernode_id_key_valid_duration_seconds: u64 =
            COVERNODE_ID_KEY_VALID_DURATION_SECONDS.try_into().unwrap();
        let id_near_expiry =
            now() - Duration::from_secs(covernode_id_key_valid_duration_seconds - 60 * 10);

        let covernode_id_key_pair =
            create_test_covernode_id_key_pair(id_near_expiry, provisioning_key);
        let published_covernode_id_key_pair = vec![covernode_id_key_pair.clone()];

        // This message key needs to be recent enough to avoid rotation but also have a truncated expiry date
        let msg_recent_date = now() - Duration::from_secs(3600 * 7);

        let covernode_msg_key_pair =
            create_test_covernode_msg_key_pair(msg_recent_date, covernode_id_key_pair.clone());
        let published_covernode_msg_key_pair = vec![covernode_msg_key_pair.clone()];

        // Sanity check - ensure the msg key has the same expiry as the parent ID key
        assert_eq!(
            covernode_msg_key_pair.key_pair.public_key().not_valid_after,
            covernode_id_key_pair.key_pair.public_key().not_valid_after
        );

        let candidate_msg_public_key = create_test_candidate_covernode_msg_key_pair(
            &published_covernode_id_key_pair,
            &published_covernode_msg_key_pair,
        );
        assert!(
            candidate_msg_public_key.is_none(),
            "Expected no key rotation for recently created key"
        );
    }
}
