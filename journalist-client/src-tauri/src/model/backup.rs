use serde::Serialize;
use ts_rs::TS;

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BackupChecks {
    pub is_backup_volume_mounted: bool,
    pub is_encrypted: bool,
    pub maybe_existing_backups: Option<Vec<String>>,
}

#[derive(Clone, Serialize, Debug, PartialEq, TS)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[ts(export)]
pub enum BackupAttemptFailureReason {
    VaultTooLarge,
    InsufficientRecoveryContactsSelected,
    InsufficientRecoveryContactsWithValidKeys,
    S3,
    Unknown,
}
