use async_trait::async_trait;
use chrono::Duration;

use super::Task;

#[derive(Default)]
pub struct HeartbeatTask {}

#[async_trait]
impl Task for HeartbeatTask {
    fn name(&self) -> &'static str {
        "heartbeat"
    }

    async fn run(&self) -> anyhow::Result<()> {
        metrics::counter!("Heartbeat").increment(1);
        Ok(())
    }

    fn interval(&self) -> Duration {
        Duration::seconds(30)
    }
}
