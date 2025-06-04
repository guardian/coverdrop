use async_trait::async_trait;
use chrono::Duration;
use common::{task::Task, time};

use crate::key_state::KeyState;

pub struct DeleteExpiredKeysTask {
    interval: Duration,
    key_state: KeyState,
}

impl DeleteExpiredKeysTask {
    pub fn new(interval: Duration, key_state: KeyState) -> Self {
        Self {
            interval,
            key_state,
        }
    }
}

#[async_trait]
impl Task for DeleteExpiredKeysTask {
    fn name(&self) -> &'static str {
        "delete_expired_keys"
    }

    async fn run(&self) -> anyhow::Result<()> {
        let now = time::now();

        let mut key_state = self.key_state.write().await;

        key_state.delete_expired_id_key_pairs(now).await?;
        key_state.delete_expired_msg_key_pairs(now).await?;

        Ok(())
    }

    fn interval(&self) -> Duration {
        self.interval
    }
}
