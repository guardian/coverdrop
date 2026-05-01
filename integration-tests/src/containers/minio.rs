use testcontainers::{runners::AsyncRunner, ContainerAsync, ImageExt};

use crate::images::{Minio, MinioArgs};

pub async fn start_minio(network: &str) -> ContainerAsync<Minio> {
    Minio::default()
        .with_cmd(MinioArgs::new().into_cmd())
        .with_network(network)
        .start()
        .await
        .expect("Start minio container")
}
