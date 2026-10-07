use std::time::Duration;
use testcontainers::{ContainerAsync, ImageExt};

use crate::containers::start_with_retry::start_with_retry;
use crate::images::{Garage, GarageArgs};

pub async fn start_garage(network: &str) -> ContainerAsync<Garage> {
    start_with_retry("Garage", || {
        Garage::default()
            .with_cmd(GarageArgs::new().into_cmd())
            .with_network(network)
            .with_startup_timeout(Duration::from_secs(60))
    })
    .await
    .expect("Start garage container")
}
