use std::time::Duration;

use common::{throttle::Throttle, time::now};

use crate::canary_state::CanaryState;

/// Create and send cloudwatch metrics for undelivered U2J and J2U messages every 5 minutes.
/// A message is considered undelivered if it was sent
/// - more than `max_delivery_time_hours` ago
/// - more recently than the duration of the validity of a journalist messaging
///   key (`JOURNALIST_MSG_KEY_VALID_DURATION_SECONDS`).
pub async fn create_undelivered_message_metrics(
    canary_state: CanaryState,
    max_delivery_time_hours: u64,
) -> anyhow::Result<()> {
    tracing::info!(
        "Starting alerts service max_delivery_time_hours={}",
        max_delivery_time_hours
    );

    // select metric value every 5 minutes
    let mut throttle = Throttle::new(Duration::from_secs(5 * 60));

    loop {
        let current_time = now();
        let undelivered_messages_result = canary_state
            .db
            .get_undelivered_messages(current_time, max_delivery_time_hours)
            .await;

        match undelivered_messages_result {
            Ok((undelivered_u2j_messages, undelivered_j2u_messages)) => {
                tracing::info!(
                    "undelivered u2j {} undelivered j2u {}",
                    undelivered_u2j_messages,
                    undelivered_j2u_messages
                );
                metrics::gauge!("UndeliveredU2JMessages").set(undelivered_u2j_messages as f64);
                metrics::gauge!("UndeliveredJ2UMessages").set(undelivered_j2u_messages as f64);
            }
            Err(err) => {
                tracing::error!(
                    "could not find number of undelivered_u2j_messages {:?}",
                    err
                )
            }
        }

        throttle.wait().await;
    }
}
