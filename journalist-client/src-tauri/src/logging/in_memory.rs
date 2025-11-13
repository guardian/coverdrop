use chrono::{DateTime, Utc};
use journalist_vault::logging::LogEntry;
use std::cmp::min;
use std::str::FromStr;
use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
};
use tokio::sync::{mpsc::UnboundedReceiver, oneshot};
use tracing::Level;

// To shut down the log buffer we send a message asking to shut down.
// That message contains another sender which allows the async task to bundle
// up the messages and send the message receiver to be reused with the vault
// based log receiver
type ShutdownConfirmationData = (Vec<LogEntry>, UnboundedReceiver<LogEntry>);
type ShutdownConfirmationSender = oneshot::Sender<ShutdownConfirmationData>;
type ShutdownSender = oneshot::Sender<ShutdownConfirmationSender>;

/// An in memory log buffer used for the period before a user opens their vault.
/// Should only ever contain a few hundred log entries at most with a low append rate.
pub struct InMemoryLogBuffer {
    // Note: I did some experimentation with some lockfree data structures but honestly
    // we don't do much logging before the vault is open so I think a RwLock around a queue is fine
    entries: Arc<RwLock<VecDeque<LogEntry>>>,
    shutdown_tx: Option<ShutdownSender>,
}

impl InMemoryLogBuffer {
    pub fn new(mut rx: UnboundedReceiver<LogEntry>) -> Self {
        let entries = Arc::new(RwLock::new(VecDeque::new()));
        let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<ShutdownConfirmationSender>();

        tauri::async_runtime::spawn({
            let entries = entries.clone();

            async move {
                loop {
                    tokio::select! {
                        Ok(confirmation_tx) = &mut shutdown_rx => {
                            let entries = entries.read().unwrap_or_else(|e| e.into_inner());
                            let data = entries.iter().cloned().collect();
                            let _ = confirmation_tx.send((data, rx));
                            break;
                        }
                        Some(log_entry) = rx.recv() => {
                            let mut entries = entries.write().unwrap_or_else(|e| e.into_inner());
                            entries.push_back(log_entry);
                        }
                    }
                }
            }
        });

        Self {
            shutdown_tx: Some(shutdown_tx),
            entries,
        }
    }

    pub async fn shutdown(&mut self) -> anyhow::Result<ShutdownConfirmationData> {
        let (confirmation_tx, confirmation_rx) = oneshot::channel();

        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            if shutdown_tx.send(confirmation_tx).is_err() {
                anyhow::bail!("Failed to send shutdown command to in-memory logger");
            }

            Ok(confirmation_rx.await?)
        } else {
            anyhow::bail!("In memory logging has already been shut down")
        }
    }

    pub fn clone_entries(
        &self,
        min_level: String,
        search_term: String,
        before: DateTime<Utc>,
        limit: usize,
        offset: usize,
    ) -> Vec<LogEntry> {
        let min_level = Level::from_str(min_level.as_str()).expect("Invalid log level");
        let results = self
            .entries
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .iter()
            .rev() // Reverse to get the most recent entries first
            .filter(|entry| {
                min_level.ge(&Level::from_str(&entry.level).unwrap_or(Level::INFO))
                    && entry.timestamp.lt(&before)
                    && (search_term.is_empty()
                        || entry
                            .message
                            .to_lowercase()
                            .contains(search_term.to_lowercase().as_str())
                        || entry
                            .target
                            .to_lowercase()
                            .contains(search_term.to_lowercase().as_str()))
            })
            .cloned()
            .collect::<Vec<LogEntry>>();
        if results.len() > offset {
            let end_index = min(offset + limit, results.len());
            results[offset..end_index].to_vec()
        } else {
            Vec::new()
        }
    }
}
