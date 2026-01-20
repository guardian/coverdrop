#![allow(unreachable_code)]
#![allow(unused_variables)]
use crate::{
    app_state::PublicInfo,
    model::{AlertLevel, BackendToFrontendEvent, BackupAttemptFailureReason},
};
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use common::{
    api::api_client::ApiClient,
    backup::constants::BACKUP_DATA_MAX_SIZE_BYTES,
    protocol::{
        backup::{sentinel_create_backup, sentinel_put_backup_data_to_s3, RecoveryContact},
        constants::{SECRET_SHARING_K_VALUE, SECRET_SHARING_N_VALUE},
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

const NUM_FAILED_ATTEMPTS_BEFORE_ALERT: u32 = 5;

#[derive(Clone)]
pub struct BackupManager {
    // Prevents concurrent backups. This might be overkill at the moment since backups are triggered by the
    // task runner which doesn't execute tasks concurrently.
    // It will be necessary if we decide to trigger a backup elsewhere, e.g. after a forced key rotation.
    lock: Arc<Mutex<()>>,
    num_failed_attempts: Arc<Mutex<u32>>,
}

impl BackupManager {
    pub fn new() -> Self {
        Self {
            lock: Arc::new(Mutex::new(())),
            num_failed_attempts: Arc::new(Mutex::new(0)),
        }
    }

    async fn increment_failed_attempts(
        &self,
        app_handle: &AppHandle,
        failure_reason: BackupAttemptFailureReason,
    ) -> anyhow::Result<()> {
        let mut failed_attempts = self.num_failed_attempts.lock().await;
        *failed_attempts += 1;
        tracing::info!(
            "Automated backup failed attempt count: {}",
            *failed_attempts
        );
        if *failed_attempts >= NUM_FAILED_ATTEMPTS_BEFORE_ALERT {
            app_handle.emit_manual_backup_required_event(Some(failure_reason))?;
            tracing::warn!(
                "Automated backup has failed {} times, manual backup required",
                *failed_attempts
            );
        }
        Ok(())
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
            // tell frontend we're starting the backup
            app_handle.emit_automated_backup_started_event()?;

            let backup_result = self
                .perform_backup_inner(app_handle, api_client, vault, vault_path, public_info, now)
                .await;
            match backup_result {
                Ok(failure_reason) => match failure_reason {
                    None => {
                        app_handle.emit_manual_backup_required_event(None)?;
                        let mut failed_attempts = self.num_failed_attempts.lock().await;
                        *failed_attempts = 0;
                    }
                    Some(reason) => {
                        self.increment_failed_attempts(app_handle, reason).await?;
                    }
                },
                Err(e) => {
                    tracing::error!("Automated backup failed: {:?}", e);
                    self.increment_failed_attempts(app_handle, BackupAttemptFailureReason::Unknown)
                        .await?;
                }
            }
            // tell frontend the backup is complete
            app_handle.emit_automated_backup_completed_event()?;
            Ok(())
        } else {
            tracing::info!("Automated backup already in progress, skipping this run");
            Ok(())
        }
    }

    /// Performs the actual backup process,
    /// Returns Ok(None) if the backup was successful,
    /// Ok(Some(reason)) if the backup failed for a known reason,
    async fn perform_backup_inner(
        &self,
        app_handle: &AppHandle,
        api_client: &ApiClient,
        vault: &JournalistVault,
        vault_path: &Path,
        public_info: &PublicInfo,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Option<BackupAttemptFailureReason>> {
        // Check the size of the vault.
        let vault_metadata = fs::metadata(vault_path)?;
        let vault_size_bytes = vault_metadata.len() as usize;
        let vault_size_warning_threshold_percentage = 80;

        // If it is greater than BACKUP_DATA_MAX_SIZE_BYTES, tell the user that the backup can't proceed.
        if vault_size_bytes > BACKUP_DATA_MAX_SIZE_BYTES {
            tracing::warn!(
                "Vault size ({}) exceeds maximum backup size ({}).",
                vault_size_bytes,
                BACKUP_DATA_MAX_SIZE_BYTES
            );
            return Ok(Some(BackupAttemptFailureReason::VaultTooLarge));
        // If it is greater than BACKUP_DATA_MAX_SIZE_BYTES * 80%, warn the user, but proceed with the backup.
        } else if vault_size_bytes
            > (BACKUP_DATA_MAX_SIZE_BYTES * vault_size_warning_threshold_percentage / 100)
        {
            app_handle.emit_alert_event(
                AlertLevel::Warning,
                "Vault size is approaching maximum backup size.",
            )?;
        }

        let public_info = public_info.get().await;
        if let Some(public_info) = public_info.as_ref() {
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

            let num_recovery_contacts_selected = recovery_contact_journalist_ids.len();
            let num_recovery_contacts_with_keys = recovery_contacts.len();
            if num_recovery_contacts_selected < SECRET_SHARING_K_VALUE {
                tracing::warn!(
                    "Fewer than {} recovery contacts found when trying to create backup.",
                    SECRET_SHARING_K_VALUE
                );
                return Ok(Some(
                    BackupAttemptFailureReason::InsufficientRecoveryContactsSelected,
                ));
            } else if num_recovery_contacts_with_keys < SECRET_SHARING_K_VALUE {
                tracing::warn!(
                    "Fewer than {} recovery contacts with valid keys found when trying to create backup.",
                    SECRET_SHARING_K_VALUE
                );
                return Ok(Some(
                    BackupAttemptFailureReason::InsufficientRecoveryContactsWithValidKeys,
                ));
            } else if num_recovery_contacts_with_keys < SECRET_SHARING_N_VALUE {
                tracing::info!(
                    "Number of recovery contacts with valid keys ({}) is less than the total number of shares to create ({}) but greater than the number required to restore ({}). Continuing with backup.",
                    num_recovery_contacts_with_keys,
                    SECRET_SHARING_N_VALUE,
                    SECRET_SHARING_K_VALUE
                );
            } else {
                tracing::info!(
                    "Found keys for all {} recovery contacts for backup.",
                    num_recovery_contacts_with_keys
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

            let put_backup_data_result = sentinel_put_backup_data_to_s3(
                api_client,
                &journalist_identity_key,
                verified_backup_data,
                now,
            )
            .await;
            if let Err(e) = put_backup_data_result {
                tracing::error!("Failed to upload backup data to S3: {:?}", e);
                return Ok(Some(BackupAttemptFailureReason::S3));
            }

            // Record successful backup
            vault
                .record_automated_backup(now, recovery_contact_journalist_ids)
                .await?;

            app_handle.emit_automated_backup_completed_event()?;
        }

        tracing::info!("Automated backup complete");

        Ok(None)
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
        Duration::minutes(1)
    }
}
