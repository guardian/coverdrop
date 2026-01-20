use chrono::Duration;

/// Lives as long as the organization key - we don't want to be rotating this super often
pub const BACKUP_ID_KEY_VALID_DURATION: Duration = Duration::weeks(52);

/// Lives as long as the organization key - we don't want to be rotating this super often
pub const BACKUP_MSG_KEY_VALID_DURATION: Duration = Duration::weeks(52);

pub const BACKUP_DATA_MAX_SIZE_BYTES: usize = 300 * 1024 * 1024; // 300 MB

pub const BACKUP_BUCKET_NAME_PREFIX: &str = "journalist-vault-backups-";
