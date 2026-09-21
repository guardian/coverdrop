use super::{record_u2c_metric_failure, record_u2c_metric_success};
use crate::checkpoint::UserToJournalistDeadDropContentWithCheckpointsAndMessageHashes;
use crate::key_state::KeyState;
use crate::mixing::mixing_message_types::SeenMessageHashes;
use crate::mixing::mixing_strategy::{
    CoverDropMixingStrategy, MixingStrategy, MixingStrategyConfiguration,
};
use common::api::models::dead_drops::UserToJournalistDeadDropMessages;
use common::api::models::messages::covernode_to_journalist_message::{
    new_random_encrypted_covernode_to_journalist_message, CoverNodeToJournalistMessage,
    EncryptedCoverNodeToJournalistMessage,
};
use common::aws::kinesis::client::StreamKind;
use common::aws::kinesis::models::checkpoint::EncryptedUserToCoverNodeMessageWithCheckpointsJson;
use common::crypto::keys::signed::SignedKey;
use common::protocol::constants::USER_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN;
use common::protocol::covernode::decrypt_user_message;
use common::protocol::keys::LatestKey;
use common::protocol::recipient_tag::RECIPIENT_TAG_FOR_COVER;
use common::time;
use covernode_database::{Database, MessageHashExpiry};
use tokio::sync::mpsc;

pub struct UserToJournalistDecryptionAndMixingService {
    key_state: KeyState,
    mixing_config: MixingStrategyConfiguration,
    db: Database,
}

impl UserToJournalistDecryptionAndMixingService {
    pub fn new(
        key_state: KeyState,
        mixing_config: MixingStrategyConfiguration,
        db: Database,
    ) -> Self {
        Self {
            key_state,
            mixing_config,
            db,
        }
    }

    pub async fn run(
        &self,
        mut inbound: mpsc::Receiver<EncryptedUserToCoverNodeMessageWithCheckpointsJson>,
        outbound: mpsc::Sender<UserToJournalistDeadDropContentWithCheckpointsAndMessageHashes>,
    ) -> anyhow::Result<()> {
        let now = time::now();

        let seen_message_hashes = self
            .db
            .select_seen_message_hashes(StreamKind::UserToJournalist, now)
            .await?
            .into_iter()
            .map(|(hash, expires_at)| Ok((hash, expires_at)))
            .collect::<anyhow::Result<SeenMessageHashes>>()?;

        let mut mixing_strategy =
            CoverDropMixingStrategy::new(self.mixing_config, now, seen_message_hashes);

        loop {
            // receive message from stream service
            let recv_message = inbound.recv().await;
            let Some(message) = recv_message else {
                continue;
            };

            // Verify that the message is the expected size
            if message.message.as_ref().len() != USER_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN {
                tracing::error!(
                    "Received U2C message of unexpected size: {} bytes",
                    message.message.as_ref().len()
                );
                record_u2c_metric_failure();
                continue;
            }

            // Lock the current key state
            let key_state = self.key_state.read().await;

            let now = time::now();

            // Attempt to decrypt the outer layer of encryption using all available
            // CoverNode messaging keys
            let Some((decrypted_message, messaging_key_expiry)) = key_state
                .covernode_msg_key_pairs_for_decryption_with_rank(now)
                .find_map(|(rank, msg_key_pair)| {
                    if let Ok(decrypted_message) =
                        decrypt_user_message(msg_key_pair, &message.message)
                    {
                        record_u2c_metric_success(rank);
                        let messaging_key_expires_at =
                            MessageHashExpiry::new(msg_key_pair.not_valid_after());
                        Some((decrypted_message, messaging_key_expires_at))
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
                time::now(),
                messaging_key_expiry,
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
                        // If we're unable to find the journalist key we shouldn't create a log,
                        // since this would leak the timing of a real message.
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

            outbound
                .send(
                    UserToJournalistDeadDropContentWithCheckpointsAndMessageHashes {
                        dead_drop_content,
                        // Always checkpoint at the last consumed message. Trade-off: buffered real messages may be lost on crash
                        // if the buffer contains more than `output_size`.
                        checkpoints_json: message.checkpoints_json,
                        encryption_max_epoch: latest_covernode_msg_key_pair.epoch,
                        message_hashes: mixing_strategy_output.message_hashes,
                    },
                )
                .await?;
        }
    }
}
