use crate::checkpoint::UserToJournalistDeadDropContentWithCheckpoints;
use crate::key_state::KeyState;
use crate::mixing::mixing_strategy::{
    CoverDropMixingStrategy, MixingStrategy, MixingStrategyConfiguration,
};
use common::api::models::dead_drops::UserToJournalistDeadDropMessages;
use common::api::models::messages::covernode_to_journalist_message::{
    new_random_encrypted_covernode_to_journalist_message, CoverNodeToJournalistMessage,
    EncryptedCoverNodeToJournalistMessage,
};
use common::aws::kinesis::models::checkpoint::EncryptedUserToCoverNodeMessageWithCheckpointsJson;
use common::protocol::covernode::decrypt_user_message;
use common::protocol::keys::LatestKey;
use common::protocol::recipient_tag::RECIPIENT_TAG_FOR_COVER;
use common::time;
use tokio::sync::mpsc;

use super::{record_u2c_metric_failure, record_u2c_metric_success};

pub struct UserToJournalistDecryptionAndMixingService {
    key_state: KeyState,
    mixing_config: MixingStrategyConfiguration,
}

impl UserToJournalistDecryptionAndMixingService {
    pub fn new(key_state: KeyState, mixing_config: MixingStrategyConfiguration) -> Self {
        Self {
            key_state,
            mixing_config,
        }
    }

    pub async fn run(
        &self,
        mut inbound: mpsc::Receiver<EncryptedUserToCoverNodeMessageWithCheckpointsJson>,
        outbound: mpsc::Sender<UserToJournalistDeadDropContentWithCheckpoints>,
    ) -> anyhow::Result<()> {
        let mut mixing_strategy = CoverDropMixingStrategy::new(self.mixing_config, time::now());

        loop {
            // receive message from stream service
            let recv_message = inbound.recv().await;
            let Some(message) = recv_message else {
                continue;
            };

            // Lock the current key state
            let key_state = self.key_state.read().await;

            let now = time::now();

            // Attempt to decrypt the outer layer of encryption using all available
            // CoverNode messaging keys
            let Some(decrypted_message) = key_state
                .covernode_msg_key_pairs_for_decryption_with_rank(now)
                .find_map(|(rank, msg_key_pair)| {
                    if let Ok(decrypted_message) =
                        decrypt_user_message(msg_key_pair, &message.message)
                    {
                        record_u2c_metric_success(rank);

                        Some(decrypted_message)
                    } else {
                        None
                    }
                })
            else {
                record_u2c_metric_failure();

                continue;
            };

            let Some(mixing_strategy_output) = mixing_strategy.consume_and_check_for_new_output(
                decrypted_message,
                message.checkpoints_json.clone(),
                time::now(),
            ) else {
                // No new dead drop to publish this time
                continue;
            };

            //
            // Time to put a dead drop onto the publishing queue
            //
            let mut messages = Vec::with_capacity(mixing_strategy_output.messages.len());

            let published_covernode_msg_key_pairs = key_state.published_covernode_msg_key_pairs();
            let latest_covernode_msg_key_pair =
                published_covernode_msg_key_pairs.latest_key_required()?;

            for (recipient_tag, u2j_message) in mixing_strategy_output.messages {
                // Messages that are marked with valid-looking journalist tags, we encrypt under
                // the intended recipient's key
                if recipient_tag != RECIPIENT_TAG_FOR_COVER {
                    // Lookup the journalist key using the recipient tag
                    if let Some(latest_journalist_msg_pk) = key_state
                        .latest_journalist_msg_pk_from_recipient_tag(&recipient_tag)
                        .await
                    {
                        // Encrypt under the journalist key
                        let c2j_message = EncryptedCoverNodeToJournalistMessage::encrypt(
                            &latest_journalist_msg_pk,
                            latest_covernode_msg_key_pair.key_pair.secret_key(),
                            CoverNodeToJournalistMessage::new(u2j_message.clone()).serialize(),
                        );

                        if let Ok(c2j_message) = c2j_message {
                            messages.push(c2j_message);
                            continue;
                        }
                    } else {
                        tracing::warn!("Couldn't find journalist messaging key from recipient tag")
                    }
                }

                // At this point the message is either a filler message (RECIPIENT_TAG_FOR_COVER)
                // or the encryption to the journalist failed. In both cases, we encrypt it with
                // a randomly generated key pair to maintain that the output is always the intended
                // size.
                if let Ok(message) = new_random_encrypted_covernode_to_journalist_message(
                    &latest_covernode_msg_key_pair.key_pair,
                    u2j_message,
                ) {
                    messages.push(message);
                } else {
                    tracing::error!("Creating random message for journalist failed");
                }
            }

            let dead_drop_content = UserToJournalistDeadDropMessages { messages };

            // If the dead drop contains real messages, write the checkpoints of its latest real message.
            // Otherwise, we can write the checkpoints of the message which triggered the dead drop.
            let checkpoints_json = mixing_strategy_output
                .checkpoints_json
                .unwrap_or(message.checkpoints_json);

            outbound
                .send(UserToJournalistDeadDropContentWithCheckpoints {
                    dead_drop_content,
                    checkpoints_json,
                    encryption_max_epoch: latest_covernode_msg_key_pair.epoch,
                })
                .await?;
        }
    }
}
