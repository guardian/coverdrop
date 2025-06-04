use async_trait::async_trait;
use chrono::Duration;
use common::{protocol::constants::MINUTE_IN_SECONDS, task::Task, time};
use journalist_vault::JournalistVault;

pub struct CleanUpVault {
    vault: JournalistVault,
}

impl CleanUpVault {
    pub fn new(vault: &JournalistVault) -> Self {
        Self {
            vault: vault.clone(),
        }
    }
}

#[async_trait]
impl Task for CleanUpVault {
    fn name(&self) -> &'static str {
        "clean_up_vault"
    }

    async fn run(&self) -> anyhow::Result<()> {
        let now = time::now();
        self.vault.clean_up(now).await?;

        Ok(())
    }

    fn interval(&self) -> Duration {
        Duration::seconds(MINUTE_IN_SECONDS * 10)
    }
}
