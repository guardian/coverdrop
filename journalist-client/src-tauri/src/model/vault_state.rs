use std::path::PathBuf;

use serde::Serialize;
use ts_rs::TS;

/// Information about the open vault that the frontend cares about
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase", tag = "type")]
#[ts(export)]
pub struct VaultState {
    pub id: String,
    pub path: PathBuf,
    pub is_soft_locked: bool,
}
