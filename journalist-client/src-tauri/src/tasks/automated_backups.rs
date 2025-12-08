#![allow(unreachable_code)]
#![allow(unused_variables)]
use crate::{
    app_state::PublicInfo,
    model::{AlertLevel, BackendToFrontendEvent},
};
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use common::{
    api::api_client::ApiClient,
    backup::constants::BACKUP_DATA_MAX_SIZE_BYTES,
    protocol::{
        backup::{sentinel_create_backup, sentinel_put_backup_data_to_s3, RecoveryContact},
        constants::{MINUTE_IN_SECONDS, SECRET_SHARING_K_VALUE, SECRET_SHARING_N_VALUE},
    },
    task::Task,
    time,
};
use journalist_vault::JournalistVault;
use std::result::Result::Ok;
use std::sync::Arc;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tauri::AppHandle;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct BackupManager {
    // Prevents concurrent backups. This might be overkill at the moment since backups are triggered by the
    // task runner which doesn't execute tasks concurrently.
    // It will be necessary if we decide to trigger a backup elsewhere, e.g. after a forced key rotation.
    lock: Arc<Mutex<()>>,
}

impl BackupManager {
    pub fn new() -> Self {
        Self {
            lock: Arc::new(Mutex::new(())),
        }
    }

    pub async fn perform_backup(
        &self,
        app_handle: &AppHandle,
        api_client: &ApiClient,
        vault: &JournalistVault,
        vault_path: &Path,
        public_info: &PublicInfo,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        // Try to acquire the lock
        if let Ok(_guard) = self.lock.try_lock() {
            self.perform_backup_inner(app_handle, api_client, vault, vault_path, public_info, now)
                .await?;
            Ok(())
        } else {
            tracing::info!("Automated backup already in progress, skipping this run");
            Ok(())
        }
    }

    async fn perform_backup_inner(
        &self,
        app_handle: &AppHandle,
        api_client: &ApiClient,
        vault: &JournalistVault,
        vault_path: &Path,
        public_info: &PublicInfo,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        // Check the size of the vault.
        let vault_metadata = fs::metadata(vault_path)?;
        let vault_size_bytes = vault_metadata.len() as usize;
        let vault_size_warning_threshold_percentage = 80;

        // If it is greater than BACKUP_DATA_MAX_SIZE_BYTES, tell the user that the backup can't proceed.
        if vault_size_bytes > BACKUP_DATA_MAX_SIZE_BYTES {
            app_handle.emit_alert_event(
                AlertLevel::Error,
                "Automated backup failed: Vault size exceeds maximum backup size. Please contact the I&R team.",
            )?;
            return Ok(());
        // If it is greater than BACKUP_DATA_MAX_SIZE_BYTES * 80%, warn the user, but proceed with the backup.
        } else if vault_size_bytes
            > (BACKUP_DATA_MAX_SIZE_BYTES * vault_size_warning_threshold_percentage / 100)
        {
            app_handle.emit_alert_event(
                AlertLevel::Warning,
                "Vault size is approaching maximum backup size.",
            )?;
        }

        // TODO instead of / in addition to bailing, we should create a metric? Or show the user?
        let public_info = public_info.get().await;
        if let Some(public_info) = public_info.as_ref() {
            // tell frontend we're starting the backup
            app_handle.emit_automated_backup_started_event()?;

            let encrypted_vault = fs::read(vault_path)?;
            let journalist_identity = vault.journalist_id().await?;
            let journalist_identity_key =
                vault.latest_id_key_pair(now).await?.ok_or_else(|| {
                    anyhow::anyhow!("No identity key found when trying to create backup")
                })?;
            let backup_admin_encryption_key =
                public_info.keys.latest_backup_msg_pk().ok_or_else(|| {
                    anyhow::anyhow!(
                        "No backup admin encryption key found when trying to create backup"
                    )
                })?;

            let recovery_contact_journalist_ids = vault.get_backup_contacts().await?;
            let recovery_contacts = recovery_contact_journalist_ids
                .iter()
                .flat_map(|id| {
                    let latest_messaging_key = public_info.keys.latest_journalist_msg_pk(id);
                    latest_messaging_key.map(|latest_messaging_key| RecoveryContact {
                        identity: id.clone(),
                        latest_messaging_key: latest_messaging_key.clone(),
                    })
                })
                .collect::<Vec<_>>();

            let num_recovery_contacts = recovery_contacts.len();
            if num_recovery_contacts < SECRET_SHARING_K_VALUE {
                // TODO show user a notification when we roll out automated backups.
                app_handle.emit_automated_backup_completed_event()?;
                // TODO raise the level of this error once the feature is no longer behind dev mode.
                // TODO also, distinguish between these two cases
                // 1. fewer than K contacts set
                // 2. fewer than K contacts with valid messaging keys
                tracing::info!(
                    "Fewer than {} recovery contacts found when trying to create backup.",
                    SECRET_SHARING_K_VALUE
                );
                return Ok(());
            } else if num_recovery_contacts < SECRET_SHARING_N_VALUE {
                tracing::warn!(
                    "Number of recovery contacts ({}) is less than the total number of shares to create ({}).",
                    num_recovery_contacts,
                    SECRET_SHARING_N_VALUE
                );
            } else {
                tracing::info!(
                    "Found keys for all {} recovery contacts for backup.",
                    num_recovery_contacts
                );
            }

            tracing::info!("Attempting to create automated backup");
            let verified_backup_data = sentinel_create_backup(
                encrypted_vault,
                journalist_identity,
                journalist_identity_key.clone(),
                backup_admin_encryption_key,
                recovery_contacts,
                SECRET_SHARING_K_VALUE.try_into().unwrap(),
                now,
            )?;

            let signed_backup_data = verified_backup_data.clone().to_unverified().unwrap();

            // TODO inform the user after 5 failed attempts
            sentinel_put_backup_data_to_s3(
                api_client,
                &journalist_identity_key,
                verified_backup_data,
                now,
            )
            .await?;

            // Record successful backup
            vault
                .record_automated_backup(now, recovery_contact_journalist_ids)
                .await?;

            app_handle.emit_automated_backup_completed_event()?;
        }

        tracing::info!("Automated backup complete");

        Ok(())
    }
}

pub struct AutomatedBackups {
    backup_manager: BackupManager,
    app_handle: AppHandle,
    api_client: ApiClient,
    vault: JournalistVault,
    vault_path: PathBuf,
    public_info: PublicInfo,
}

impl AutomatedBackups {
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
impl Task for AutomatedBackups {
    fn name(&self) -> &'static str {
        "automated_backups"
    }

    async fn run(&self) -> anyhow::Result<()> {
        let now = time::now();

        let require_backup = self
            .vault
            .get_count_of_keys_created_since_last_backup()
            .await?
            > 0;

        if !require_backup {
            tracing::info!("No new keys since last backup, skipping automated backup");
            return Ok(());
        }

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

        Ok(())
    }

    fn interval(&self) -> Duration {
        Duration::seconds(MINUTE_IN_SECONDS)
    }
}
