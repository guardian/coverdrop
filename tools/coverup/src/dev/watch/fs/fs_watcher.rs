use std::path::PathBuf;

use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::broadcast::{self, Receiver, Sender};

use crate::dev::build::cargo::CargoMetadata;

use super::{FsSignal, FsWatchedCrate};

pub struct FsWatcher {
    // Handle must be held because dropping this
    // closes the event watchers
    inner_watcher: RecommendedWatcher,
    paths: Vec<PathBuf>,
    fs_signal_tx: Sender<FsSignal>,
}

impl FsWatcher {
    pub fn new(cargo_metadata: &CargoMetadata) -> anyhow::Result<Self> {
        let paths = cargo_metadata.workspace_member_paths();
        let watched_crates = FsWatchedCrate::from_cargo_metadata(cargo_metadata);

        if watched_crates.is_empty() {
            anyhow::bail!("No crates to watch, are you in a coverdrop project directory?");
        }

        let (notify_tx, _) = broadcast::channel::<FsSignal>(100);

        let fs_signal_tx = notify_tx.clone();

        let inner_watcher = notify::recommended_watcher(
            move |res: Result<notify::Event, notify::Error>| match res {
                Ok(event) => match event.kind {
                    EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_) => {
                        tracing::debug!("Notify edit event occured at {:?}", event.paths);

                        for path in event.paths {
                            let Some(matching_crate) =
                                watched_crates.iter().max_by_key(|watched_crate| {
                                    watched_crate
                                        .path
                                        .components()
                                        .zip(path.components())
                                        .take_while(|(a, b)| a == b)
                                        .count()
                                })
                            else {
                                tracing::warn!(
                                    "No matching watched crate found for {}",
                                    path.display()
                                );
                                return;
                            };

                            tracing::info!(
                                "Modification event found in crate {}",
                                matching_crate.name
                            );

                            for service in &matching_crate.rebuild_on_change {
                                let signal = FsSignal::Dirty(*service);

                                if let Err(e) = notify_tx.send(signal) {
                                    tracing::error!(
                                        "Failed to send build task {:?} to builder: {:?}",
                                        service,
                                        e
                                    );
                                    continue;
                                }
                            }
                        }
                    }
                    _ => {
                        tracing::debug!(
                            "Got presumed uninteresting event from notify: {:?}",
                            event.kind
                        )
                    }
                },
                Err(e) => tracing::error!("notify watch error: {:?}", e),
            },
        )?;

        let watcher = Self {
            paths,
            inner_watcher,
            fs_signal_tx,
        };

        Ok(watcher)
    }

    pub fn subscribe(&self) -> Receiver<FsSignal> {
        self.fs_signal_tx.subscribe()
    }

    /// Start watching the paths. Thread pool is managed externally so this
    /// doesn't need to have it's lifetime managed by a tokio task.
    pub fn watch(&mut self) -> anyhow::Result<()> {
        for path in &self.paths {
            tracing::debug!("Starting watcher for {}", path.display());

            self.inner_watcher.watch(path, RecursiveMode::Recursive)?;
        }

        Ok(())
    }
}
