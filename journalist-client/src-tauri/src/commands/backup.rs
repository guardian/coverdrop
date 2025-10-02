use crate::app_state::AppStateHandle;
use crate::error::{CommandError, GenericSnafu, IoSnafu, VaultLockedSnafu, VaultSnafu};
use crate::model::{BackupChecks, VaultState};
use common::time;
use snafu::{OptionExt, ResultExt};
use std::path::PathBuf;
use std::{fs, process};
use tauri::State;

pub const BACKUP_VOLUME_PATH: &str = "/Volumes/SentinelBackup";

#[tauri::command]
pub async fn should_require_backup(app: State<'_, AppStateHandle>) -> Result<bool, CommandError> {
    let vault = app.inner().vault().await.context(VaultLockedSnafu)?;

    Ok(vault
        .get_count_of_keys_created_since_last_backup()
        .await
        .context(VaultSnafu {
            failed_to: "check whether a backup is required",
        })?
        > 0)
}

#[tauri::command]
pub async fn get_backup_checks() -> Result<BackupChecks, CommandError> {
    tracing::debug!("get backup checks");
    let volume_info_output = process::Command::new("diskutil")
        .arg("info")
        .arg(BACKUP_VOLUME_PATH)
        .output()
        .context(IoSnafu {
            failed_to: "check backup volume encryption status",
        })?;

    tracing::debug!("diskutil command exited with {}", volume_info_output.status);
    if !volume_info_output.status.success() {
        tracing::debug!("The backup volume either doesn't exist or hasn't been mounted.");
        return Ok(BackupChecks {
            is_backup_volume_mounted: false,
            is_encrypted: false,
            maybe_existing_backups: None,
        });
    }

    let is_encrypted = String::from_utf8_lossy(&volume_info_output.stdout)
        .lines()
        .any(|line| line.trim().starts_with("FileVault:") && line.trim().ends_with("Yes"));

    let files_on_backup_volume = fs::read_dir(BACKUP_VOLUME_PATH).context(IoSnafu {
        failed_to: "read files on backup volume",
    })?;

    let mut existing_backups: Vec<_> = files_on_backup_volume
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                let path = e.path();
                if path.extension()?.to_str()? == "backup" {
                    Some(path.file_name()?.to_str()?.to_string())
                } else {
                    None
                }
            })
        })
        .collect();
    existing_backups.sort_by(|a, b| b.cmp(a));
    Ok(BackupChecks {
        is_backup_volume_mounted: true,
        is_encrypted,
        maybe_existing_backups: Some(existing_backups),
    })
}

#[tauri::command]
pub async fn perform_backup(app: State<'_, AppStateHandle>) -> Result<(), CommandError> {
    let vault = app.inner().vault().await.context(VaultLockedSnafu)?;

    let vault_state: VaultState = app
        .inner()
        .vault_state()
        .await
        .context(VaultSnafu {
            failed_to: "get vault state",
        })?
        .context(GenericSnafu {
            ctx: "vault state is None",
        })?;

    let backup_checks_double_check = get_backup_checks().await;

    if !backup_checks_double_check?.is_encrypted {
        return Err(GenericSnafu {
            ctx: "backup checks failed",
        }
        .build());
    }

    let now = time::now();

    let backup_path_buf = PathBuf::from(BACKUP_VOLUME_PATH).join(format!(
        "{}__{}.backup",
        vault_state.id,
        now.to_rfc3339()
    ));
    let backup_path = backup_path_buf.as_path().to_str().context(GenericSnafu {
        ctx: "construct backup path",
    })?;

    fs::copy(vault_state.path, backup_path).context(IoSnafu {
        failed_to: "copy vault to backup volume",
    })?;

    vault
        .record_successful_backup(now, backup_path)
        .await
        .context(VaultSnafu {
            failed_to: "track successful backup in DB",
        })?;

    Ok(())
}

#[tauri::command]
pub async fn eject_backup_volume() -> Result<bool, CommandError> {
    let eject_output = process::Command::new("diskutil")
        .arg("eject")
        .arg(BACKUP_VOLUME_PATH)
        .output()
        .context(IoSnafu {
            failed_to: "eject backup volume",
        })?;

    tracing::debug!("diskutil command exited with {}", eject_output.status);

    Ok(eject_output.status.success())
}
