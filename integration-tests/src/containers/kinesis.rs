use std::time::Duration;
use testcontainers::{runners::AsyncRunner, ContainerAsync, ImageExt};

use crate::images::Kinesis;

pub async fn start_kinesis(network: &str) -> ContainerAsync<Kinesis> {
    Kinesis::default()
        .with_network(network)
        .with_startup_timeout(Duration::from_secs(60))
        .start()
        .await
        .expect("Start kinesis container")
}
