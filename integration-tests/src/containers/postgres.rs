use std::time::Duration;
use testcontainers::{ContainerAsync, ImageExt};

use crate::containers::start_with_retry::start_with_retry;
use crate::images::{Postgres, PostgresArgs};

pub async fn start_postgres(network: &str) -> ContainerAsync<Postgres> {
    start_with_retry("Postgres", || {
        Postgres::default()
            .with_cmd(PostgresArgs::new().into_cmd())
            .with_network(network)
            .with_startup_timeout(Duration::from_secs(60))
    })
    .await
    .expect("Start postgres container")
}
