use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::api::models::sentinel_id::SentinelIdentity;

#[derive(Clone, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct SentinelProfile {
    pub id: SentinelIdentity,
    pub display_name: String,
}

impl SentinelProfile {
    pub fn new(id: SentinelIdentity, display_name: String) -> Self {
        Self { id, display_name }
    }
}
