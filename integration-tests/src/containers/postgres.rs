use testcontainers::{runners::AsyncRunner, ContainerAsync, ImageExt};

use crate::images::{Postgres, PostgresArgs};

pub async fn start_postgres(network: &str) -> ContainerAsync<Postgres> {
    Postgres::default()
        .with_cmd(PostgresArgs::new().into_cmd())
        .with_network(network)
        .start()
        .await
        .expect("Start postgres container")
}
