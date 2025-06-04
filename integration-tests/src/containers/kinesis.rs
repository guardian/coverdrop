use testcontainers::{runners::AsyncRunner, ContainerAsync, RunnableImage};

use crate::images::Kinesis;

pub async fn start_kinesis(network: &str) -> ContainerAsync<Kinesis> {
    let kinesis_image = RunnableImage::from(Kinesis::default());

    kinesis_image
        .with_network(network)
        .start()
        .await
        .expect("Start kinesis container")
}
