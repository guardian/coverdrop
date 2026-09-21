use std::time::Duration;
use testcontainers::{ContainerAsync, ImageExt};

use crate::containers::start_with_retry::start_with_retry;
use crate::images::{Minio, MinioArgs};

pub async fn start_minio(network: &str) -> ContainerAsync<Minio> {
    start_with_retry("MinIO", || {
        Minio::default()
            .with_cmd(MinioArgs::new().into_cmd())
            .with_network(network)
            .with_startup_timeout(Duration::from_secs(60))
    })
    .await
    .expect("Start minio container")
}
