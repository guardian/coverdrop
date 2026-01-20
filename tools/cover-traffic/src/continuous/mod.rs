pub mod message_per_hour;

use chrono::{DateTime, Utc};
use common::api::api_client::ApiClient;
use common::aws::kinesis::client::KinesisClient;
use common::throttle::Throttle;
use common::time::now;
use common::tracing::log_task_exit;
use common::u2j_appender::messaging_client::MessagingClient;
use message_per_hour::MessagesPerHour;
use std::future::Future;
use std::ops::Div;
use std::sync::Arc;
use std::time::Duration;

use crate::state::CoverTrafficState;

const KEY_UPDATE_INTERVAL: chrono::Duration = chrono::Duration::seconds(60);
const TRAFFIC_STATISTICS_OUTPUT_INTERVAL: chrono::Duration = chrono::Duration::seconds(15);
const UPDATE_MPH_INTERVAL: chrono::Duration = chrono::Duration::minutes(5);

pub async fn send_cover_traffic_continuously(
    api_client: ApiClient,
    messaging_client: MessagingClient,
    kinesis_client: KinesisClient,
    state: CoverTrafficState,
    mph_u2j: MessagesPerHour,
    mph_j2u: MessagesPerHour,
) -> anyhow::Result<()> {
    tracing::debug!("mph_u2j={}, mph_j2u={}", mph_u2j.value(), mph_j2u.value());

    let shared_state = Arc::new(state);

    // The key hierarchy update service downloads the latest public key endpoint content frequently
    // and updates the [CoverDropSharedState] accordingly.
    let mut key_hierarchy_update_service = tokio::spawn({
        let api_client = api_client.clone();
        let shared_state = shared_state.clone();

        async move {
            let mut throttle = Throttle::new(KEY_UPDATE_INTERVAL.to_std().unwrap());

            loop {
                throttle.wait().await;
                let untrusted_keys_and_profiles = api_client
                    .get_public_keys()
                    .await
                    .expect("Download public keys");
                shared_state
                    .update_keys(untrusted_keys_and_profiles.keys)
                    .await;
                tracing::info!("Updated the key hierarchy");
            }
        }
    });

    // This cover traffic service sends cover messages to the user-facing side of the
    // CoverNode based on the CLI rate argument.
    let mut cover_traffic_user_to_covernode_service = tokio::spawn({
        let messaging_client = messaging_client.clone();
        let shared_state = shared_state.clone();

        async move {
            let mut output_rate_controller =
                OutputRateController::new("U2J cover traffic", mph_u2j);

            loop {
                output_rate_controller
                    .wait_and_execute(|| async {
                        let msg = shared_state
                            .create_user_to_journalist_cover_message()
                            .await
                            .expect("Create U2J cover message");
                        messaging_client
                            .post_user_message(msg)
                            .await
                            .expect("Send U2J message");
                    })
                    .await;
            }
        }
    });

    // This cover traffic service sends cover messages to the journalist-facing side of the
    // CoverNode based on the CLI rate argument.
    let mut cover_traffic_journalist_to_covernode_service = tokio::spawn({
        let kinesis_client = kinesis_client.clone();
        let shared_state = shared_state.clone();

        async move {
            let mut output_rate_controller =
                OutputRateController::new("J2U cover traffic", mph_j2u);

            loop {
                output_rate_controller
                    .wait_and_execute(|| async {
                        let msg = shared_state
                            .create_journalist_to_user_cover_message()
                            .await
                            .expect("Create J2U cover message");
                        kinesis_client
                            .encode_and_put_journalist_message(msg)
                            .await
                            .expect("Send J2U message");
                    })
                    .await;
            }
        }
    });

    tokio::select! {
        r = (&mut key_hierarchy_update_service) => {
            log_task_exit("key hierarchy update", r);

            cover_traffic_user_to_covernode_service.abort();
            cover_traffic_journalist_to_covernode_service.abort();
        },
        r = (&mut cover_traffic_user_to_covernode_service) => {
            log_task_exit("cover traffic user to covernode", r);

            key_hierarchy_update_service.abort();
            cover_traffic_journalist_to_covernode_service.abort();
        },
        r = (&mut cover_traffic_journalist_to_covernode_service) => {
            log_task_exit("cover traffic journalist to covernode", r);

            key_hierarchy_update_service.abort();
            cover_traffic_user_to_covernode_service.abort();
        },
    }

    Ok(())
}

/// The [OutputRateController] controls the output rate using [Throttle] and outputs statistics
/// of the handled events regularly.
struct OutputRateController {
    messages_per_hour: MessagesPerHour,
    name: &'static str,
    start_time: DateTime<Utc>,
    last_output_statistics: DateTime<Utc>,
    last_updated_mph: DateTime<Utc>,
    total_events: u64,
    throttle: Throttle,
}

impl OutputRateController {
    fn new(name: &'static str, messages_per_hour: MessagesPerHour) -> Self {
        let throttle = Self::create_throttle(name, &messages_per_hour);

        OutputRateController {
            messages_per_hour,
            name,
            start_time: now(),
            last_output_statistics: now(),
            last_updated_mph: now(),
            total_events: 0,
            throttle,
        }
    }

    fn create_throttle(name: &str, mph: &MessagesPerHour) -> Throttle {
        let mph = *mph.value();

        if mph == 0 {
            // If we're sending zero messages then we need to still poll for new MPH settings occasionally
            Throttle::new(Duration::from_secs(60))
        } else {
            let average_durations = Duration::from_secs(3600).div(mph);
            tracing::debug!(
                "Will execute '{}' every {:.2} seconds",
                name,
                average_durations.as_secs_f32()
            );

            Throttle::new(average_durations)
        }
    }

    async fn update_messages_per_second(&mut self) {
        if let Err(e) = self.messages_per_hour.update().await {
            tracing::error!("Failed to update messages per second: {}", e);
        }

        self.throttle = Self::create_throttle(self.name, &self.messages_per_hour);
    }

    async fn wait_and_execute<F, Fut>(&mut self, f: F)
    where
        F: Fn() -> Fut,
        Fut: Future<Output = ()>,
    {
        // Wait (if necessary) until next expected target time
        self.throttle.wait().await;

        // Execute the target operation - special casing if we're emitting zero messages
        // per hour
        if *self.messages_per_hour.value() > 0 {
            f().await;
            self.total_events += 1;
        }

        // Output statistics
        if now() - self.last_output_statistics > TRAFFIC_STATISTICS_OUTPUT_INTERVAL {
            let total_running_duration = now() - self.start_time;
            tracing::info!(
                "Executed '{}' {} times within {} seconds",
                self.name,
                self.total_events,
                total_running_duration.num_seconds()
            );
            self.last_output_statistics = now()
        }

        // Check if we need to change our messages per hour
        if now() - self.last_updated_mph > UPDATE_MPH_INTERVAL {
            self.update_messages_per_second().await;

            self.last_updated_mph = now()
        }
    }
}
