mod from_journalist_polling_service;
mod from_user_polling_service;

use std::time::Duration;

use common::throttle::AdaptiveThrottle;
pub use from_journalist_polling_service::FromJournalistPollingService;
pub use from_user_polling_service::FromUserPollingService;

pub type KinesisPollerThrottle = AdaptiveThrottle<usize>;

pub fn new_kinesis_poller_throttle(target_batch_size_per_shard: usize) -> KinesisPollerThrottle {
    // Kinesis only allows 5 calls to GetRecords per second, given that we need to briefly have
    // two CoverNodes during rolling updates we can play it safe by maxing out at 2 per second.
    const MIN_SLEEP_DURATION: Duration = Duration::from_millis(500);

    // Don't sleep for more than 5 seconds. If the Kinesis stream is running dry (e.g. in local dev)
    // we don't want to just wait for an ever increasingly long amount of time
    const MAX_SLEEP_DURATION: Duration = Duration::from_millis(5000);

    // Adapt the sleep duration in 50ms increments.
    const ADAPTION_DURATION: Duration = Duration::from_millis(50);

    AdaptiveThrottle::<usize>::new(
        // Start at the max duration, to slow start the poller. If we started with a smaller duration we risk
        // overloading the shard query limits during a scale up.
        MAX_SLEEP_DURATION,
        MIN_SLEEP_DURATION,
        MAX_SLEEP_DURATION,
        ADAPTION_DURATION,
        target_batch_size_per_shard,
    )
}
