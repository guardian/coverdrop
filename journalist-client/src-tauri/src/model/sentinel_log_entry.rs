use chrono::{DateTime, Utc};
use journalist_vault::logging::LogEntry;
use serde::Serialize;
use tracing::Level;
use ts_rs::TS;

// Basically a mirror for the journalist_vault::LogEntry type but with the
// appropriate annotations to work with Tauri and our TypeScript
//
// Maybe we should add the TS library to the journalist_vault directly but
// that feels like it's pulling down concerns from the client.

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SentinelLogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: i32,
    pub target: String,
    pub message: String,
}

impl SentinelLogEntry {
    pub fn from_log_entry(log_entry: &LogEntry) -> Self {
        // Use a numeric representation of the log level to make filtering easier on the client
        // If you change this you must update the corresponding map in the LogsPanel.tsx file
        let level = match log_entry.level {
            Level::TRACE => 0,
            Level::DEBUG => 1,
            Level::INFO => 2,
            Level::WARN => 3,
            Level::ERROR => 4,
        };

        Self {
            timestamp: log_entry.timestamp,
            level,
            target: log_entry.target.clone(),
            message: log_entry.message.clone(),
        }
    }
}
