//! This crate is used only for integration testing
extern crate covernode as covernode_app;

pub mod api_wrappers;
pub mod constants;
pub mod containers;
pub mod docker_utils;
pub mod images;
pub mod keys;
pub mod mailboxes;
pub mod panic_handler;
pub mod secrets;
pub mod stack;
pub mod utils;
pub mod vectors;
use std::{future::Future, thread::sleep, time::Duration};

pub use images::{dev_j2u_mixing_config, dev_u2j_mixing_config};
pub use stack::CoverDropStack;

// Generic retry for async functions
pub async fn retry_async<T, E, R, F>(sleep_duration: Duration, attempts: usize, func: F) -> T
where
    F: Fn() -> R,
    R: Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    for _ in 0..attempts {
        match func().await {
            Ok(v) => return v,
            Err(e) => {
                tracing::warn!("Failed attempt in retry loop: {:?}", e);
                sleep(sleep_duration);
            }
        }
    }

    panic!(
        "Failed to successfully run function after {} retries",
        attempts
    );
}
