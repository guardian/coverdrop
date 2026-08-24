use crate::checkpoint::JournalistToUserDeadDropContentWithCheckpoints;
use crate::mixing::mixing_strategy::{
    CoverDropMixingStrategy, MixingStrategy, MixingStrategyConfiguration,
};

use crate::key_state::KeyState;
use common::api::models::dead_drops::JournalistToUserDeadDropMessages;
use common::aws::kinesis::models::checkpoint::EncryptedJournalistToCoverNodeMessageWithCheckpointsJson;
use common::protocol::constants::JOURNALIST_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN;
use common::protocol::covernode::decrypt_journalist_message;
use common::time;
use tokio::sync::mpsc;

use super::{record_j2c_metric_failure, record_j2c_metric_success};

pub struct JournalistToUserDecryptionAndMixingService {
    key_state: KeyState,
    mixing_config: MixingStrategyConfiguration,
}

impl JournalistToUserDecryptionAndMixingService {
    pub fn new(
        keys: KeyState,
        mixing_config: MixingStrategyConfiguration,
    ) -> JournalistToUserDecryptionAndMixingService {
        JournalistToUserDecryptionAndMixingService {
            key_state: keys,
            mixing_config,
        }
    }

    pub async fn run(
        &self,
        mut inbound: mpsc::Receiver<EncryptedJournalistToCoverNodeMessageWithCheckpointsJson>,
        outbound: mpsc::Sender<JournalistToUserDeadDropContentWithCheckpoints>,
    ) -> anyhow::Result<()> {
        let mut mixing_strategy = CoverDropMixingStrategy::new(self.mixing_config, time::now());

        loop {
            // receive message from stream service
            let recv_message = inbound.recv().await;
            let Some(message) = recv_message else {
                continue;
            };

            // Verify that the message is the expected size
            if message.message.as_ref().len() != JOURNALIST_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN {
                tracing::error!(
                    "Received J2C message of unexpected size: {} bytes",
                    message.message.as_ref().len()
                );
                record_j2c_metric_failure();
                continue;
            }

            let key_state = self.key_state.read().await;

            let now = time::now();

            let Some(decrypted_message) = key_state
                .covernode_msg_key_pairs_for_decryption_with_rank(now)
                .find_map(|(rank, covernode_msg_key_pair)| {
                    if let Ok(decrypted_message) =
                        decrypt_journalist_message(covernode_msg_key_pair, &message.message)
                    {
                        record_j2c_metric_success(rank);
                        Some(decrypted_message)
                    } else {
                        None
                    }
                })
            else {
                record_j2c_metric_failure();

                continue;
            };

            let Some(mixing_strategy_output) =
                mixing_strategy.consume_and_check_for_new_output(decrypted_message, time::now())
            else {
                // No new epoch to publish this time
                continue;
            };

            // Always checkpoint at the last consumed message. Trade-off: buffered real messages may be lost on crash
            // if the buffer contains more than `output_size`.
            let checkpoints_json = message.checkpoints_json;

            // handle new epoch
            outbound
                .send(JournalistToUserDeadDropContentWithCheckpoints {
                    dead_drop_content: JournalistToUserDeadDropMessages {
                        messages: mixing_strategy_output.messages,
                    },
                    checkpoints_json,
                })
                .await?;
        }
    }
}
