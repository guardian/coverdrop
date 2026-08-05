use std::path::PathBuf;

use serde::Serialize;
use ts_rs::TS;

/// Information about the open vault that the frontend cares about
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase", tag = "type")]
#[ts(export)]
pub struct VaultState {
    // This will eventually be removed once sentinel id to journalist id
    // is one-to-many.
    pub journalist_id: String,
    pub sentinel_id: Option<String>,
    pub path: PathBuf,
    pub is_soft_locked: bool,
}
