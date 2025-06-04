use crate::protocol::constants::WEEK_IN_SECONDS;

/// Lives as long as the organization key - we don't want to be rotating this super often
pub const ADMIN_KEY_VALID_DURATION_SECONDS: i64 = 52 * WEEK_IN_SECONDS;
