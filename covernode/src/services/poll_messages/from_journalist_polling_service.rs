use std::time::Duration;

use common::aws::kinesis::client::KinesisClient;
use common::aws::kinesis::models::checkpoint::EncryptedJournalistToCoverNodeMessageWithCheckpointsJson;

use common::time;
use tokio::sync::mpsc;
use tokio::time::sleep;

use super::new_kinesis_poller_throttle;

const POLLING_BATCH_SIZE_PER_SHARD: i32 = 1000;
const TARGET_BATCH_SIZE_PER_SHARD: usize = 5;

pub struct FromJournalistPollingService {
    kinesis_client: KinesisClient,
    disable_stream_throttle: bool,
}

impl FromJournalistPollingService {
    pub fn new(
        kinesis_client: KinesisClient,
        disable_stream_throttle: bool,
    ) -> FromJournalistPollingService {
        FromJournalistPollingService {
            kinesis_client,
            disable_stream_throttle,
        }
    }

    pub async fn run(
        &mut self,
        outbound: mpsc::Sender<EncryptedJournalistToCoverNodeMessageWithCheckpointsJson>,
    ) -> anyhow::Result<()> {
        let mut throttle = new_kinesis_poller_throttle(TARGET_BATCH_SIZE_PER_SHARD);

        let mut error_count = 0;

        loop {
            let now = time::now();

            let messages = self
                .kinesis_client
                .read_journalist_messages(POLLING_BATCH_SIZE_PER_SHARD, now)
                .await;

            let messages_sent = match messages {
                Ok(messages) => {
                    error_count = 0;

                    let num_messages = messages.len();

                    tracing::info!(num_messages, "Got new J2U messages");

                    metrics::counter!("J2UMessagesFromKinesis").increment(num_messages as u64);

                    // pass messages along
                    for message in messages {
                        match message {
                            Ok(message) => {
                                outbound
                                    .send(message)
                                    .await
                                    .expect("Outbound channel receiver open");
                            }
                            Err(e) => tracing::error!(
                                "Error in parsing journalist message from kinesis: {e}"
                            ),
                        }
                    }

                    num_messages
                }
                Err(err) => {
                    error_count += 1;

                    if error_count >= 3 {
                        // We've seen behavior where the CoverNode gets stuck in a loop of polling failed
                        // until a reboot - so just panic and let kubernetes pick it back up.
                        //
                        // This is far from ideal since if we are in a position where we panic consistently
                        // before a dead drop is released then we will never move our checkpoint forward and
                        // the system as a whole will stall.
                        panic!("Polling journalist messages failed: {err:?}");
                    }

                    0
                }
            };

            if !self.disable_stream_throttle {
                if let Some(time_slept) = throttle.wait(messages_sent).await {
                    metrics::histogram!("J2UPollingThrottleTimeSeconds")
                        .record(time_slept.as_secs_f64());
                }
            } else {
                sleep(Duration::from_millis(100)).await;
            }
        }
    }
}
