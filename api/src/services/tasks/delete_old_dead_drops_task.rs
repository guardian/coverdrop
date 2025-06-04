use async_trait::async_trait;
use chrono::Duration;
use common::{task::Task, time};

use crate::services::database::Database;

pub struct DeleteOldDeadDropsTask {
    interval: Duration,
    db: Database,
}

impl DeleteOldDeadDropsTask {
    pub fn new(interval: Duration, db: Database) -> Self {
        Self { interval, db }
    }
}

#[async_trait]
impl Task for DeleteOldDeadDropsTask {
    fn name(&self) -> &'static str {
        "delete_old_dead_drops"
    }

    async fn run(&self) -> anyhow::Result<()> {
        self.db
            .dead_drop_queries
            .delete_old_dead_drops(time::now())
            .await?;

        Ok(())
    }

    fn interval(&self) -> Duration {
        self.interval
    }
}
