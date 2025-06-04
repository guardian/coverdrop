use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::dead_drops::DeadDropId;

/// Summary information about dead drops. Useful to debugging
/// manually and used by the vault to skip past dead drops that
/// cannot possibly contain messages during setup.
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeadDropSummary {
    pub id: DeadDropId,
    pub created_at: DateTime<Utc>,
}
