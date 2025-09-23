use tauri::{AppHandle, Emitter};
use ts_rs::TS;

#[derive(TS)]
#[ts(export, rename_all = "snake_case")]
enum EventType {
    OutboundQueueLength,
    DeadDropsRemaining,
}

impl EventType {
    fn as_str(&self) -> &'static str {
        match self {
            EventType::OutboundQueueLength => "outbound_queue_length",
            EventType::DeadDropsRemaining => "dead_drops_remaining",
        }
    }
}

pub trait BackendToFrontendEvent {
    fn emit_outbound_queue_length_event(&self, length: i32) -> anyhow::Result<()>;

    fn emit_dead_drops_pull_started(&self) -> anyhow::Result<()>;
    fn emit_dead_drops_remaining_event(&self, count: usize) -> anyhow::Result<()>;
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
}
