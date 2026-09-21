use async_trait::async_trait;
use chrono::Duration;
use common::{task::Task, time};
use covernode_database::Database;

pub struct DeleteExpiredSeenMessageHashesTask {
    interval: Duration,
    db: Database,
}

impl DeleteExpiredSeenMessageHashesTask {
    pub fn new(interval: Duration, db: Database) -> Self {
        Self { interval, db }
    }
}

#[async_trait]
impl Task for DeleteExpiredSeenMessageHashesTask {
    fn name(&self) -> &'static str {
        "delete_expired_seen_message_hashes"
    }

    async fn run(&self) -> anyhow::Result<()> {
        let now = time::now();

        self.db.delete_expired_seen_message_hashes(now).await?;

        Ok(())
    }

    fn interval(&self) -> Duration {
        self.interval
    }
}
