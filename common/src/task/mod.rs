mod client;
mod heartbeat_task;
mod runner;

pub use client::TaskApiClient;
pub use heartbeat_task::HeartbeatTask;
pub use runner::{RunnerMode, TaskRunner, TASK_RUNNER_API_PORT};

use async_trait::async_trait;
use chrono::Duration;

#[async_trait]
pub trait Task {
    fn name(&self) -> &'static str;
    async fn run(&self) -> anyhow::Result<()>;
    fn interval(&self) -> Duration;
}
