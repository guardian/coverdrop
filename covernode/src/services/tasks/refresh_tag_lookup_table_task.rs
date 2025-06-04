use async_trait::async_trait;
use chrono::Duration;
use common::{task::Task, time};

use crate::key_state::KeyState;

pub struct RefreshTagLookUpTableTask {
    interval: Duration,
    key_state: KeyState,
}

impl RefreshTagLookUpTableTask {
    pub fn new(interval: Duration, key_state: KeyState) -> Self {
        Self {
            interval,
            key_state,
        }
    }
}

#[async_trait]
impl Task for RefreshTagLookUpTableTask {
    fn name(&self) -> &'static str {
        "refresh_tag_look_up_table"
    }

    async fn run(&self) -> anyhow::Result<()> {
        let now = time::now();

        self.key_state
            .write()
            .await
            .refresh_tag_lookup_table(now)
            .await
    }

    fn interval(&self) -> Duration {
        self.interval
    }
}
