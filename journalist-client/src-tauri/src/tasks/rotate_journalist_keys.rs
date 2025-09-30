use crate::model::BackendToFrontendEvent;
use async_trait::async_trait;
use chrono::Duration;
use common::{api::api_client::ApiClient, protocol::constants::HOUR_IN_SECONDS, task::Task, time};
use journalist_vault::JournalistVault;
use tauri::AppHandle;

pub struct RotateJournalistKeys {
    app_handle: AppHandle,
    api_client: ApiClient,
    vault: JournalistVault,
}

impl RotateJournalistKeys {
    pub fn new(app_handle: &AppHandle, api_client: &ApiClient, vault: &JournalistVault) -> Self {
        Self {
            app_handle: app_handle.clone(),
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

        let did_rotate_some_keys = self
            .vault
            .check_and_rotate_keys(&self.api_client, now)
            .await?;

        if did_rotate_some_keys {
            self.app_handle.emit_journalist_keys_rotated_event()?;
        }

        Ok(())
    }

    fn interval(&self) -> Duration {
        Duration::seconds(HOUR_IN_SECONDS)
    }
}
