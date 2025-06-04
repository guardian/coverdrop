use std::{env, io::Stdout};
use tokio::task::JoinError;
use tracing_subscriber::{
    filter::Filtered,
    fmt::{
        format::{DefaultFields, Format, Full, Json, JsonFields},
        Layer,
    },
    prelude::*,
    reload::{self, Handle},
    EnvFilter, Layer as TracingLayer, Registry,
};

/// Initializes a new tracing configuration.
///
/// - `rust_log`: Used to set the RUST_LOG environment variable if it is not provided. You can set the default log level (e.g. `warn`),
///   but you can also configure module-specific log levels using comma-separated entries formatted like `path::to::module=log_level`, e.g.
///   `warn,test::foo=info,test::foo::bar=debug`
pub fn init_tracing(rust_log: &str) {
    // Just use the with handle version but ignore the handle
    init_tracing_with_reload_handle(rust_log);
}

// We support two types of logging so our reload handles can be different types.
// Hence there are some involved type definitions here.

type FilteredLayer<T, U> =
    Filtered<Layer<Registry, T, Format<U>, fn() -> Stdout>, EnvFilter, Registry>;

// Used in production
type JsonFilteredLayer = FilteredLayer<JsonFields, Json>;

// Used when we don't want JSON logs
type StandardFilteredLayer = FilteredLayer<DefaultFields, Full>;

#[derive(Clone)]
pub enum TracingReloadHandle {
    Json(Handle<JsonFilteredLayer, Registry>),
    Text(Handle<StandardFilteredLayer, Registry>),
}

impl TracingReloadHandle {
    pub fn update(&self, rust_log: &str) -> anyhow::Result<()> {
        match self {
            TracingReloadHandle::Json(h) => {
                h.modify(|l| *l.filter_mut() = EnvFilter::new(rust_log))?
            }
            TracingReloadHandle::Text(h) => {
                h.modify(|l| *l.filter_mut() = EnvFilter::new(rust_log))?
            }
        }

        Ok(())
    }
}

/// Initializes a new tracing configuration with a reload handle.
///
/// The reload handle can be used to modify the logging directive after
/// the application has started.
///
/// For more details on the parameters see `init_tracing`
pub fn init_tracing_with_reload_handle(rust_log: &str) -> TracingReloadHandle {
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", rust_log);
    }

    let json_logs = env::var_os("JSON_LOGS").is_some();

    // Conditionally output logs as JSON
    if json_logs {
        let layer = Layer::new()
            .with_writer(std::io::stdout as fn() -> Stdout)
            .with_target(true)
            .json()
            .flatten_event(true)
            .with_span_list(false)
            .with_filter(EnvFilter::from_default_env());

        let (layer, reload_handle) = reload::Layer::new(layer);
        tracing_subscriber::registry().with(layer).init();

        TracingReloadHandle::Json(reload_handle)
    } else {
        let layer = Layer::new()
            .with_writer(std::io::stdout as fn() -> Stdout)
            .with_target(true)
            .with_filter(EnvFilter::from_default_env());
        let (layer, reload_handle) = reload::Layer::new(layer);
        tracing_subscriber::registry().with(layer).init();

        TracingReloadHandle::Text(reload_handle)
    }
}

pub fn log_task_result_exit<T, E>(task_name: &'static str, result: Result<Result<T, E>, JoinError>)
where
    E: std::fmt::Debug,
{
    match result {
        Ok(Ok(_)) => tracing::info!("Task '{}' exited successfully", task_name),
        Ok(Err(e)) => tracing::error!("Failure in '{}' task: {:?}", task_name, e),
        Err(e) => tracing::error!("Failed to join to '{}' task handle: {:?}", task_name, e),
    }
}

pub fn log_task_exit(task_name: &'static str, result: Result<(), JoinError>) {
    match result {
        Ok(()) => tracing::info!("Task '{}' exited successfully", task_name),
        Err(e) => tracing::error!("Failed to join to '{}' task handle: {:?}", task_name, e),
    }
}
