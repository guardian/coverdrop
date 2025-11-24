use std::path::{Path, PathBuf};

use crate::{app_state::PublicInfo, model::BackendToFrontendEvent, tasks::BackupManager};
use async_trait::async_trait;
use chrono::Duration;
use common::{api::api_client::ApiClient, protocol::constants::HOUR_IN_SECONDS, task::Task, time};
use journalist_vault::JournalistVault;
use tauri::AppHandle;

pub struct RotateJournalistKeys {
    backup_manager: BackupManager,
    app_handle: AppHandle,
    api_client: ApiClient,
    vault: JournalistVault,
    vault_path: PathBuf,
    public_info: PublicInfo,
}

impl RotateJournalistKeys {
    pub fn new(
        backup_manager: &BackupManager,
        app_handle: &AppHandle,
        api_client: &ApiClient,
        vault: &JournalistVault,
        vault_path: &Path,
        public_info: &PublicInfo,
    ) -> Self {
        Self {
            backup_manager: backup_manager.clone(),
            app_handle: app_handle.clone(),
            api_client: api_client.clone(),
            vault: vault.clone(),
            vault_path: vault_path.to_path_buf(),
            public_info: public_info.clone(),
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

            tracing::info!("Rotated journalist keys, performing automated backup");
            self.backup_manager
                .perform_backup(
                    &self.app_handle,
                    &self.api_client,
                    &self.vault,
                    &self.vault_path,
                    &self.public_info,
                    now,
                )
                .await?;
        }

        Ok(())
    }

    fn interval(&self) -> Duration {
        Duration::seconds(HOUR_IN_SECONDS)
    }
}
