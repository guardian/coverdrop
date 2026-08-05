use std::time::Duration;
use testcontainers::{runners::AsyncRunner, ContainerAsync, ImageExt};

use crate::images::{Minio, MinioArgs};

pub async fn start_minio(network: &str) -> ContainerAsync<Minio> {
    Minio::default()
        .with_cmd(MinioArgs::new().into_cmd())
        .with_network(network)
        .with_startup_timeout(Duration::from_secs(60))
        .start()
        .await
        .expect("Start minio container")
}
