use std::{
    cmp::{max, min},
    time::{Duration, Instant},
};

pub struct AdaptiveThrottle<T: Ord> {
    min_sleep_duration: Duration,
    max_sleep_duration: Duration,

    adaption_duration: Duration,

    current_sleep_duration: Duration,

    next_instant: Instant,

    target_volume_per_poll: T,
}

impl<T> AdaptiveThrottle<T>
where
    T: Ord,
{
    pub fn new(
        initial_sleep_duration: Duration,
        min_sleep_duration: Duration,
        max_sleep_duration: Duration,
        adaption_duration: Duration,
        target_volume_per_poll: T,
    ) -> Self {
        assert!(max_sleep_duration > min_sleep_duration);

        Self {
            min_sleep_duration,
            max_sleep_duration,
            adaption_duration,
            current_sleep_duration: initial_sleep_duration,
            next_instant: Instant::now(),
            target_volume_per_poll,
        }
    }

    /// Wait for the appropriate amount of time given the batch size.
    /// Returns the time slept, if any.
    pub async fn wait(&mut self, batch_size: T) -> Option<Duration> {
        if batch_size < self.target_volume_per_poll {
            // If we are lower than our target batch size, wait a bit more time
            self.current_sleep_duration = min(
                self.current_sleep_duration
                    .saturating_add(self.adaption_duration),
                self.max_sleep_duration,
            );
        } else {
            // If we are higher than our target batch size, wait a bit less time
            self.current_sleep_duration = max(
                self.current_sleep_duration
                    .saturating_sub(self.adaption_duration),
                self.min_sleep_duration,
            );
        }

        let duration_until_next_instant = self.next_instant.checked_duration_since(Instant::now());

        if let Some(sleep_duration) = duration_until_next_instant {
            tokio::time::sleep(sleep_duration).await;
        }

        self.next_instant = Instant::now() + self.current_sleep_duration;

        duration_until_next_instant
    }
}

pub struct Throttle {
    target_rate: Duration,
    next_instant: Instant,
}

impl Throttle {
    pub fn new(target_rate: Duration) -> Throttle {
        Throttle {
            target_rate,
            next_instant: Instant::now(),
        }
    }

    pub async fn wait(&mut self) {
        let duration_until_next_instant = self.next_instant.checked_duration_since(Instant::now());
        if let Some(sleep_duration) = duration_until_next_instant {
            tokio::time::sleep(sleep_duration).await;
        }

        // schedule end of next wait
        self.next_instant = Instant::now() + self.target_rate;
    }
}

/// Simple back off delay, multiplies a base delay time
/// by some number of "misses". For example if your base delay is 500ms
/// the delay would progress:
///
/// 500ms, 1000ms, 1500ms ...
pub struct BackOffDelay {
    base_delay: Duration,
    max_delay: Duration,

    misses: u32,
}

impl BackOffDelay {
    pub fn new(base_delay: Duration, max_delay: Duration) -> Self {
        BackOffDelay {
            base_delay,
            max_delay,
            misses: 0,
        }
    }

    pub async fn wait(&mut self) -> Option<Duration> {
        self.maybe_wait(true).await
    }

    /// Wait if you missed, returns if a sleep happened
    pub async fn maybe_wait(&mut self, should_sleep: bool) -> Option<Duration> {
        if should_sleep {
            self.misses += 1;
            let mut sleep_duration = self.base_delay * self.misses;

            if sleep_duration < self.max_delay {
                sleep_duration = self.max_delay;
            };

            tokio::time::sleep(sleep_duration).await;

            Some(sleep_duration)
        } else {
            self.misses = 0;
            None
        }
    }
}
