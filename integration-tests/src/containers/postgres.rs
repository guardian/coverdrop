use std::time::Duration;
use testcontainers::{runners::AsyncRunner, ContainerAsync, ImageExt};

use crate::images::{Postgres, PostgresArgs};

pub async fn start_postgres(network: &str) -> ContainerAsync<Postgres> {
    Postgres::default()
        .with_cmd(PostgresArgs::new().into_cmd())
        .with_network(network)
        .with_startup_timeout(Duration::from_secs(60))
        .start()
        .await
        .expect("Start postgres container")
}
