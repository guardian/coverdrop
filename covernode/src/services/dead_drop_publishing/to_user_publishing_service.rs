use crate::checkpoint::JournalistToUserDeadDropContentWithCheckpoints;
use crate::key_state::KeyState;
use crate::update_checkpoint;
use common::api::api_client::ApiClient;
use common::api::models::dead_drops::{
    JournalistToUserDeadDropSignatureDataV2, UnpublishedJournalistToUserDeadDrop,
};
use common::aws::kinesis::client::StreamKind;
use common::protocol::keys::LatestKey;
use common::throttle::BackOffDelay;
use common::time;

use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc;

pub struct ToUserPublishingService {
    key_state: KeyState,
    api_client: ApiClient,
    checkpoint_path: PathBuf,
}

// Delay for backoff when we fail to publish a dead drop
const BASE_DELAY_MS: u64 = 1_000;
const MAX_DELAY_MS: u64 = 60_000;

impl ToUserPublishingService {
    pub fn new(keys: KeyState, api_client: ApiClient, checkpoint_path: PathBuf) -> Self {
        Self {
            key_state: keys,
            api_client,
            checkpoint_path,
        }
    }

    pub async fn run(
        &self,
        mut inbound: mpsc::Receiver<JournalistToUserDeadDropContentWithCheckpoints>,
    ) -> anyhow::Result<()> {
        loop {
            let inbound = inbound
                .recv()
                .await
                .expect("Receive message on inbound message queue");

            tracing::debug!("Got a new Journalist->User dead drop");

            let key_state = self.key_state.read().await;

            let published_covernode_id_key_pairs = key_state.published_covernode_id_key_pairs();

            let Ok(latest_id_key_pair) = published_covernode_id_key_pairs.latest_key_required()
            else {
                tracing::error!("No CoverNode identity key available, cannot continue!");
                panic!("No CoverNode identity key available, cannot continue!");
            };

            let dead_drop = inbound.dead_drop_content;
            let serialized_dead_drop = dead_drop.serialize();

            // V1 signature
            let certificate = latest_id_key_pair.key_pair.sign(&serialized_dead_drop);

            // V2 signature
            let created_at = time::now();
            let signature_data =
                JournalistToUserDeadDropSignatureDataV2::new(&serialized_dead_drop, created_at);
            let signature = latest_id_key_pair.key_pair.sign(&signature_data);

            let mut delay = BackOffDelay::new(
                Duration::from_millis(BASE_DELAY_MS),
                Duration::from_millis(MAX_DELAY_MS),
            );

            let signed_dead_drop = UnpublishedJournalistToUserDeadDrop::new(
                serialized_dead_drop,
                created_at,
                certificate,
                signature,
            );

            // Loop until we successfully publish the dead drop.
            // Waiting with a back off between failed attempts
            loop {
                let post_dead_drop_attempt =
                    self.api_client.post_user_dead_drop(&signed_dead_drop).await;

                match post_dead_drop_attempt {
                    Ok(()) => {
                        tracing::info!("Successfully posted J2U dead drop");
                        break;
                    }
                    Err(e) => {
                        tracing::error!("Failed to publish J2U dead drop: {:?}", e);

                        if let Some(slept_for) = delay.wait().await {
                            tracing::trace!(
                                "Slept J2U dead drop publish backoff for {}ms",
                                slept_for.as_millis()
                            )
                        }
                    }
                }
            }

            tracing::info!("Saving J2U checkpoints: {:?}", inbound.checkpoints_json);

            if let Err(e) = update_checkpoint(
                self.checkpoint_path.clone(),
                StreamKind::JournalistToUser,
                inbound.checkpoints_json,
            ) {
                // If the CoverNode crashes between now and publishing the next checkpoint we will
                // possibly republish dead drops.
                tracing::error!("Failed to update CoverNode checkpoint: {}", e);
            };
        }
    }
}
