use std::{fmt, sync::Arc};

use chrono::{DateTime, Utc};
use common::time;
use in_memory::InMemoryLogBuffer;
use journalist_vault::{logging::LogEntry, JournalistVault};
use tokio::sync::{mpsc::UnboundedSender, RwLock};
use tracing::{
    field::{Field, Visit},
    Event, Subscriber,
};
use tracing_subscriber::{layer::Context, Layer};
use vault::VaultLogger;

mod in_memory;
mod vault;

pub enum LogReceiverTarget {
    InMemory(InMemoryLogBuffer),
    Vault(VaultLogger),
}

struct InnerLogReceiver {
    target: LogReceiverTarget,
}

#[derive(Clone)]
pub struct LogReceiver {
    tx: UnboundedSender<LogEntry>,
    data: Arc<RwLock<InnerLogReceiver>>,
}

impl Default for LogReceiver {
    fn default() -> Self {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let target = LogReceiverTarget::InMemory(InMemoryLogBuffer::new(rx));

        Self {
            tx,
            data: Arc::new(RwLock::new(InnerLogReceiver { target })),
        }
    }
}

impl LogReceiver {
    pub async fn use_vault(
        &self,
        vault: &JournalistVault,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        // Copy the vault handle so we can manage a copy from this structure
        let vault = vault.clone();

        let mut inner = self.data.write().await;

        let target = &mut inner.target;

        if let LogReceiverTarget::InMemory(in_memory_log_buffer) = target {
            let (log_entries, rx) = in_memory_log_buffer.shutdown().await?;

            let vault_logger = VaultLogger::new(vault, rx, now).await?;

            for log_entry in log_entries {
                if let Err(e) = self.tx.send(log_entry) {
                    eprintln!("Failed to send log entry: {}", e);
                }
            }

            *target = LogReceiverTarget::Vault(vault_logger);
        }

        Ok(())
    }

    /// Get all the log entries from this session
    pub async fn get_entries(&self) -> anyhow::Result<Vec<LogEntry>> {
        let (session_id, vault) = {
            let inner = self.data.read().await;

            let target = &inner.target;

            match target {
                LogReceiverTarget::InMemory(in_memory_log_buffer) => {
                    return Ok(in_memory_log_buffer.clone_entries());
                }
                LogReceiverTarget::Vault(vault_logger) => vault_logger.get_vault(),
            }
        };

        vault.get_log_entries(session_id).await
    }
}

pub struct JournalistClientLogLayer {
    log_receiver: LogReceiver,
}

impl JournalistClientLogLayer {
    pub fn new(log_receiver: LogReceiver) -> Self {
        Self { log_receiver }
    }
}

struct MessageVisitor<'a>(&'a mut String);

impl Visit for MessageVisitor<'_> {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if field.name() == "message" {
            self.0.push_str(&format!("{:?}", value));
        }
    }
}

impl<S: Subscriber> Layer<S> for JournalistClientLogLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let metadata = event.metadata();

        // Extract message from event fields
        let mut message = String::new();
        let mut visitor = MessageVisitor(&mut message);
        event.record(&mut visitor);

        // Create log entry
        let log_entry = LogEntry::new(time::now(), *metadata.level(), metadata.target(), message);

        if let Err(e) = self.log_receiver.tx.send(log_entry) {
            eprintln!("Failed to send log entry: {}", e);
        }
    }
}
