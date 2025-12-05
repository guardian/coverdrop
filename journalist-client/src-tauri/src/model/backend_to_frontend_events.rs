use serde::Serialize;
use tauri::{AppHandle, Emitter};
use ts_rs::TS;

#[derive(TS)]
#[ts(export, rename_all = "snake_case")]
enum EventType {
    OutboundQueueLength,
    DeadDropsRemaining,
    JournalistKeysRotated,
    AutomatedBackup,
    // a generic alert / notification event
    Alert,
}

impl EventType {
    fn as_str(&self) -> &'static str {
        match self {
            EventType::OutboundQueueLength => "outbound_queue_length",
            EventType::DeadDropsRemaining => "dead_drops_remaining",
            EventType::JournalistKeysRotated => "journalist_keys_rotated",
            EventType::AutomatedBackup => "automated_backup",
            EventType::Alert => "notification",
        }
    }
}

#[derive(TS, Serialize, Clone)]
#[ts(export, rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AlertLevel {
    Warning,
    Error,
}

#[derive(TS, Serialize, Clone)]
#[ts(export)]
struct AlertPayload {
    level: AlertLevel,
    message: String,
}

pub trait BackendToFrontendEvent {
    fn emit_outbound_queue_length_event(&self, length: i32) -> anyhow::Result<()>;

    fn emit_dead_drops_pull_started(&self) -> anyhow::Result<()>;
    fn emit_dead_drops_remaining_event(&self, count: usize) -> anyhow::Result<()>;
    fn emit_journalist_keys_rotated_event(&self) -> anyhow::Result<()>;
    fn emit_automated_backup_started_event(&self) -> anyhow::Result<()>;
    fn emit_automated_backup_completed_event(&self) -> anyhow::Result<()>;

    fn emit_alert_event(&self, level: AlertLevel, message: &str) -> anyhow::Result<()>;
}

impl BackendToFrontendEvent for AppHandle {
    fn emit_outbound_queue_length_event(&self, length: i32) -> anyhow::Result<()> {
        self.emit(EventType::OutboundQueueLength.as_str(), length)?;
        Ok(())
    }

    fn emit_dead_drops_pull_started(&self) -> anyhow::Result<()> {
        self.emit(EventType::DeadDropsRemaining.as_str(), None::<i32>)?;
        Ok(())
    }

    fn emit_dead_drops_remaining_event(&self, count: usize) -> anyhow::Result<()> {
        self.emit(EventType::DeadDropsRemaining.as_str(), count)?;
        Ok(())
    }

    fn emit_journalist_keys_rotated_event(&self) -> anyhow::Result<()> {
        self.emit(EventType::JournalistKeysRotated.as_str(), None::<i32>)?;
        Ok(())
    }

    fn emit_automated_backup_started_event(&self) -> anyhow::Result<()> {
        self.emit(EventType::AutomatedBackup.as_str(), 1)?;
        Ok(())
    }

    fn emit_automated_backup_completed_event(&self) -> anyhow::Result<()> {
        self.emit(EventType::AutomatedBackup.as_str(), 0)?;
        Ok(())
    }

    fn emit_alert_event(&self, level: AlertLevel, message: &str) -> anyhow::Result<()> {
        self.emit(
            EventType::Alert.as_str(),
            AlertPayload {
                level,
                message: message.to_string(),
            },
        )?;
        Ok(())
    }
}
