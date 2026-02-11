use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// This outcome is returned when a vault is successfully unlocked
#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "type"
)]
#[ts(export)]
pub enum OpenVaultOutcome {
    OpenedOffline,
    OpenedOnline,
}
