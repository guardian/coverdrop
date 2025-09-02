use crate::checkpoint::UserToJournalistDeadDropContentWithCheckpoints;
use crate::key_state::KeyState;
use crate::update_checkpoint;
use common::api::api_client::ApiClient;
use common::api::models::dead_drops::{
    UnpublishedUserToJournalistDeadDrop, UserToJournalistDeadDropCertificateDataV1,
    UserToJournalistDeadDropSignatureDataV2,
};
use common::aws::kinesis::client::StreamKind;
use common::protocol::keys::LatestKey;
use common::throttle::BackOffDelay;
use common::time;

use std::cmp::max;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc;

pub struct ToJournalistPublishingService {
    key_state: KeyState,
    api_client: ApiClient,
    checkpoint_path: PathBuf,
}

// Delay for backoff when we fail to publish a dead drop
const BASE_DELAY_MS: u64 = 1_000;
const MAX_DELAY_MS: u64 = 60_000;

impl ToJournalistPublishingService {
    pub fn new(key_state: KeyState, api_client: ApiClient, checkpoint_path: PathBuf) -> Self {
        Self {
            key_state,
            api_client,
            checkpoint_path,
        }
    }

    pub async fn run(
        &self,
        mut inbound: mpsc::Receiver<UserToJournalistDeadDropContentWithCheckpoints>,
    ) -> anyhow::Result<()> {
        loop {
            let inbound = inbound.recv().await.unwrap();

            tracing::debug!("Got a new User->Journalist dead drop");

            let key_state = self.key_state.read().await;

            let published_covernode_id_key_pairs = key_state.published_covernode_id_key_pairs();
            let Some(latest_id_key_pair) = published_covernode_id_key_pairs.latest_key() else {
                tracing::error!("No CoverNode identity key available, cannot continue!");
                panic!("No CoverNode identity key available, cannot continue!");
            };

            let signing_max_epoch = latest_id_key_pair.epoch;
            let max_epoch = max(inbound.encryption_max_epoch, signing_max_epoch);

            let dead_drop = inbound.dead_drop_content;
            let serialized_dead_drop_messages = dead_drop.serialize();

            // V1 signature
            let certificate_data = UserToJournalistDeadDropCertificateDataV1::new(
                &serialized_dead_drop_messages,
                max_epoch,
            );
            let cert = latest_id_key_pair.key_pair.sign(&certificate_data);

            let created_at = time::now();

            // V2 signature
            let signature_data = UserToJournalistDeadDropSignatureDataV2::new(
                &serialized_dead_drop_messages,
                created_at,
                max_epoch,
            );
            let signature = latest_id_key_pair.key_pair.sign(&signature_data);

            let mut delay = BackOffDelay::new(
                Duration::from_millis(BASE_DELAY_MS),
                Duration::from_millis(MAX_DELAY_MS),
            );

            let dead_drop = UnpublishedUserToJournalistDeadDrop::new(
                serialized_dead_drop_messages,
                cert,
                signature,
                created_at,
                max_epoch,
            );

            // Loop until we successfully publish the dead drop.
            // Waiting with a back off between failed attempts
            loop {
                let post_dead_drop_attempt =
                    self.api_client.post_journalist_dead_drop(&dead_drop).await;

                match post_dead_drop_attempt {
                    Ok(()) => {
                        tracing::info!("Successfully posted U2J dead drop");
                        break;
                    }
                    Err(e) => {
                        tracing::error!("Failed to publish U2J dead drop: {:?}", e);

                        if let Some(slept_for) = delay.wait().await {
                            tracing::trace!(
                                "Slept U2J dead drop publish backoff for {}ms",
                                slept_for.as_millis()
                            )
                        }
                    }
                }
            }

            tracing::info!("Saving U2J checkpoints: {:?}", inbound.checkpoints_json);

            if let Err(e) = update_checkpoint(
                self.checkpoint_path.clone(),
                StreamKind::UserToJournalist,
                inbound.checkpoints_json,
            ) {
                // If the CoverNode crashes between now and publishing the next checkpoint we will
                // possibly republish dead drops.
                tracing::error!("Failed to update CoverNode checkpoint: {}", e);
            }
        }
    }
}
