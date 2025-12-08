use crate::protocol::constants::WEEK_IN_SECONDS;

/// Lives as long as the organization key - we don't want to be rotating this super often
pub const BACKUP_ID_KEY_VALID_DURATION_SECONDS: i64 = 52 * WEEK_IN_SECONDS;

/// Lives as long as the organization key - we don't want to be rotating this super often
pub const BACKUP_MSG_KEY_VALID_DURATION_SECONDS: i64 = 52 * WEEK_IN_SECONDS;

pub const BACKUP_DATA_MAX_SIZE_BYTES: usize = 300 * 1024 * 1024; // 300 MB

pub const BACKUP_BUCKET_NAME_PREFIX: &str = "journalist-vault-backups-";
