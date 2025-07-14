use std::time::Duration;

use chrono::{DateTime, Utc};
use journalist_vault::{logging::LogEntry, JournalistVault};
use tokio::{sync::mpsc::UnboundedReceiver, task::JoinHandle, time::interval};

pub struct VaultLogger {
    session_id: i64,
    _writer_task_handle: JoinHandle<()>,
    vault: JournalistVault,
}

impl VaultLogger {
    pub async fn new(
        vault: JournalistVault,
        mut rx: UnboundedReceiver<LogEntry>,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        let session_id = vault.add_session(now).await?;

        let _writer_task_handle = tokio::task::spawn({
            let write_handle_vault = vault.clone();

            let mut interval = interval(Duration::from_secs(1));

            let mut log_buf = Vec::<LogEntry>::with_capacity(1024);

            // 10 MiB of logs will be buffered in memory before we start dropping log entries
            // This is to prevent runaway memory usage if we're not able to flush the logs to disk
            // In the future we should add the ability to pass a flag to the JC that forwards *all*
            // logs to stdout.
            const MAX_LOGS_SIZE: usize = 10 * 1024 * 1024;
            let mut buf_size = 0;

            async move {
                loop {
                    tokio::select! {
                        Some(log_entry) = rx.recv() => {
                            let entry_size = log_entry.size();

                            if buf_size + entry_size < MAX_LOGS_SIZE {
                                log_buf.push(log_entry);
                                buf_size += entry_size;
                            } else {
                                eprintln!("Failed to queue log message to disk. Suggests there's an issue flushing logs to the vault");
                            }
                        },
                        _ = interval.tick() => {
                            match write_handle_vault
                                    .add_log_entries(session_id, &log_buf)
                                    .await {
                                Ok(_) =>  {
                                    log_buf.clear();
                                    buf_size = 0;
                                }
                                Err(e) => {
                                    eprintln!("Failed to save log entry to vault: {e:?}");
                                }
                            }
                        }
                    }
                }
            }
        });

        Ok(Self {
            session_id,
            _writer_task_handle,
            vault,
        })
    }

    pub fn get_vault(&self) -> (i64, JournalistVault) {
        (self.session_id, self.vault.clone())
    }
}
