use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// This outcome is returned when a vault is successfully unlocked but includes warnings
#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "type"
)]
#[ts(export)]
pub enum OpenVaultOutcome {
    OpenedOffline,
    OpenedOnline {
        org_pks_missing_in_vault: Vec<String>,
        org_pks_missing_in_api: Vec<String>,
    },
}
