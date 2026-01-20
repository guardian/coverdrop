use chrono::Duration;

/// Lives as long as the organization key - we don't want to be rotating this super often
pub const ADMIN_KEY_VALID_DURATION: Duration = Duration::weeks(52);
