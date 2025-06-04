use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use common::{
    protocol::{
        constants::{
            COVERNODE_ID_KEY_ROTATE_AFTER_SECONDS, COVERNODE_ID_KEY_VALID_DURATION_SECONDS,
            COVERNODE_MSG_KEY_ROTATE_AFTER_SECONDS, COVERNODE_MSG_KEY_VALID_DURATION_SECONDS,
        },
        keys::{
            generate_covernode_messaging_key_pair, generate_unregistered_covernode_id_key_pair,
            CoverNodeMessagingKeyPair, LatestKey, UnregisteredCoverNodeIdKeyPair,
        },
    },
    task::Task,
    time,
};
use tokio::sync::RwLockReadGuard;

use crate::key_state::{InnerKeyState, KeyState};

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
        key_state: &RwLockReadGuard<'_, InnerKeyState>,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Option<UnregisteredCoverNodeIdKeyPair>> {
        let published_id_key_pairs = key_state.published_covernode_id_key_pairs();

        // We have at least one valid id key pair, so we need to check if it is old
        // enough to require a rotation
        if let Some(latest_id_key_pair) = published_id_key_pairs.latest_key() {
            let key_pair_created_at = latest_id_key_pair.key_pair.public_key().not_valid_after
                - Duration::seconds(COVERNODE_ID_KEY_VALID_DURATION_SECONDS);
            let key_pair_rotate_at =
                key_pair_created_at + Duration::seconds(COVERNODE_ID_KEY_ROTATE_AFTER_SECONDS);

            tracing::info!(
                "Checking latest identity key expiry ({}) against now ({})",
                latest_id_key_pair.key_pair.public_key().not_valid_after,
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
        key_state: &RwLockReadGuard<'_, InnerKeyState>,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Option<CoverNodeMessagingKeyPair>> {
        let published_id_key_pairs = key_state.published_covernode_id_key_pairs();
        let latest_id_key_pair = published_id_key_pairs
            .latest_key_required()
            .map(|k| &k.key_pair)?;

        let msg_key_pairs = key_state.published_covernode_msg_key_pairs();

        // We have at least one valid msg key pair, so we need to check if it is old
        // enough to require a rotation
        if let Some(latest_msg_key_pair_with_epoch) = msg_key_pairs.latest_key() {
            let latest_msg_key_pair = &latest_msg_key_pair_with_epoch.key_pair;
            let key_pair_created_at = latest_msg_key_pair.public_key().not_valid_after
                - Duration::seconds(COVERNODE_MSG_KEY_VALID_DURATION_SECONDS);
            let key_pair_rotate_at =
                key_pair_created_at + Duration::seconds(COVERNODE_MSG_KEY_ROTATE_AFTER_SECONDS);

            tracing::info!(
                "Checking latest messaging key expiry ({}) against now ({})",
                latest_msg_key_pair.public_key().not_valid_after,
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
                maybe_new_id_key_pair = Self::maybe_create_candidate_id_key_pair(&key_state, now)?;
            }

            if maybe_candidate_msg_key_pair.is_none() {
                tracing::info!(
                    "No candidate messaging key found, checking if we should create one..."
                );
                maybe_new_msg_key_pair =
                    Self::maybe_create_candidate_msg_key_pair(&key_state, now)?;
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
