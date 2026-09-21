use std::time::Duration;
use testcontainers::{ContainerAsync, ImageExt};

use crate::containers::start_with_retry::start_with_retry;
use crate::images::Kinesis;

pub async fn start_kinesis(network: &str) -> ContainerAsync<Kinesis> {
    start_with_retry("Kinesis", || {
        Kinesis::default()
            .with_network(network)
            .with_startup_timeout(Duration::from_secs(60))
    })
    .await
    .expect("Start kinesis container")
}
