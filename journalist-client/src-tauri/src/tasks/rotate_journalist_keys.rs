use async_trait::async_trait;
use chrono::Duration;
use common::{api::api_client::ApiClient, protocol::constants::HOUR_IN_SECONDS, task::Task, time};
use journalist_vault::JournalistVault;

pub struct RotateJournalistKeys {
    api_client: ApiClient,
    vault: JournalistVault,
}

impl RotateJournalistKeys {
    pub fn new(api_client: &ApiClient, vault: &JournalistVault) -> Self {
        Self {
            api_client: api_client.clone(),
            vault: vault.clone(),
        }
    }
}

#[async_trait]
impl Task for RotateJournalistKeys {
    fn name(&self) -> &'static str {
        "rotate_journalist_keys"
    }

    async fn run(&self) -> anyhow::Result<()> {
        let now = time::now();

        self.vault
            .check_and_rotate_keys(&self.api_client, now)
            .await?;

        Ok(())
    }

    fn interval(&self) -> Duration {
        Duration::seconds(HOUR_IN_SECONDS)
    }
}
