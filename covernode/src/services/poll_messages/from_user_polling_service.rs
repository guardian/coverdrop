use std::time::Duration;

use common::aws::kinesis::client::KinesisClient;
use common::aws::kinesis::models::checkpoint::EncryptedUserToCoverNodeMessageWithCheckpointsJson;

use common::time;
use tokio::sync::mpsc;
use tokio::time::sleep;

use super::new_kinesis_poller_throttle;

const MAX_BATCH_SIZE_PER_SHARD: i32 = 1000;
// Tune the poll rate to get about 200 messages per batch. This needs to be lower
// than the max batch size so that the adaptive throttle can tune itself.
// With just Android traffic we see ~60 messages per second.
const TARGET_BATCH_SIZE_PER_SHARD: usize = 200;

pub struct FromUserPollingService {
    kinesis_client: KinesisClient,
    disable_stream_throttle: bool,
}

impl FromUserPollingService {
    pub fn new(
        kinesis_client: KinesisClient,
        disable_stream_throttle: bool,
    ) -> FromUserPollingService {
        FromUserPollingService {
            kinesis_client,
            disable_stream_throttle,
        }
    }

    pub async fn run(
        &mut self,
        outbound: mpsc::Sender<EncryptedUserToCoverNodeMessageWithCheckpointsJson>,
    ) -> anyhow::Result<()> {
        let mut throttle = new_kinesis_poller_throttle(TARGET_BATCH_SIZE_PER_SHARD);

        let mut error_count = 0;

        loop {
            let now = time::now();

            let messages = self
                .kinesis_client
                .read_user_messages(MAX_BATCH_SIZE_PER_SHARD, now)
                .await;

            let messages_sent = match messages {
                Ok(messages) => {
                    error_count = 0;

                    let num_messages = messages.len();

                    tracing::info!(num_messages, "Got new U2J messages");

                    metrics::counter!("U2JMessagesFromKinesis").increment(num_messages as u64);

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
                                "Error in parsing user to journalist message from kinesis: {e}"
                            ),
                        }
                    }
                    num_messages
                }
                Err(err) => {
                    error_count += 1;

                    tracing::error!(
                        "Error reading user to journalist messages from kinesis: {:?}, error count: {}, root cause: {}",
                        err,
                        error_count,
                        err.root_cause()
                    );

                    if error_count >= 10 {
                        // We've seen behavior where the CoverNode gets stuck in a loop of polling failed
                        // until a reboot - so just panic and let kubernetes pick it back up.
                        //
                        // This is far from ideal since if we are in a position where we panic consistently
                        // before a dead drop is released then we will never move our checkpoint forward and
                        // the system as a whole will stall.
                        panic!("Polling user messages failed: {err:?}");
                    }

                    0
                }
            };

            if !self.disable_stream_throttle {
                if let Some(time_slept) = throttle.wait(messages_sent).await {
                    metrics::histogram!("U2JPollingThrottleTimeSeconds")
                        .record(time_slept.as_secs_f64());
                }
            } else {
                sleep(Duration::from_millis(100)).await;
            }
        }
    }
}
