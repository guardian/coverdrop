use async_trait::async_trait;
use chrono::Duration;
use common::{task::Task, time};

use crate::services::database::Database;

pub struct DeleteOldDeduplicationIdsTask {
    interval: Duration,
    db: Database,
}

impl DeleteOldDeduplicationIdsTask {
    pub fn new(interval: Duration, db: Database) -> Self {
        Self { interval, db }
    }
}

#[async_trait]
impl Task for DeleteOldDeduplicationIdsTask {
    fn name(&self) -> &'static str {
        "delete_old_deduplication_ids"
    }

    async fn run(&self) -> anyhow::Result<()> {
        self.db
            .j2c_deduplication_id_queries
            .delete_old_deduplication_ids(time::now())
            .await?;

        Ok(())
    }

    fn interval(&self) -> Duration {
        self.interval
    }
}
