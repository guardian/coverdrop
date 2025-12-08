use testcontainers::{runners::AsyncRunner, ContainerAsync, RunnableImage};

use crate::images::{Minio, MinioArgs};

pub async fn start_minio(network: &str) -> ContainerAsync<Minio> {
    let minio_image = RunnableImage::from((Minio::default(), MinioArgs::new()));

    minio_image
        .with_network(network)
        .start()
        .await
        .expect("Start minio container")
}
