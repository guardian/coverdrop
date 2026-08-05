use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

/// Uniquely identifies a message in an MLS group.
/// Also used as a deduplication token so that the API can prevent duplicate J2C messages.
/// We use the same type for both purposes in anticipation of future changes where Sentinel
/// will forward MLS messages to sources via the CoverNode. Multiple members of a shared profile
/// MLS group may attempt to forward the same message, and the API should deduplicate them.
#[derive(TS)]
#[ts(export)]
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, sqlx::Type)]
#[sqlx(transparent)]
pub struct MessageId(#[ts(type = "string")] Uuid);

impl Default for MessageId {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Display for MessageId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
