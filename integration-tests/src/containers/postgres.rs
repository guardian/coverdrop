use testcontainers::{runners::AsyncRunner, ContainerAsync, RunnableImage};

use crate::images::{Postgres, PostgresArgs};

pub async fn start_postgres(network: &str) -> ContainerAsync<Postgres> {
    let postgres_image = RunnableImage::from((Postgres::default(), PostgresArgs::new()));

    postgres_image
        .with_network(network)
        .start()
        .await
        .expect("Start postgres container")
}
