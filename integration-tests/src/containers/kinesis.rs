use testcontainers::{runners::AsyncRunner, ContainerAsync, ImageExt};

use crate::images::Kinesis;

pub async fn start_kinesis(network: &str) -> ContainerAsync<Kinesis> {
    Kinesis::default()
        .with_network(network)
        .start()
        .await
        .expect("Start kinesis container")
}
