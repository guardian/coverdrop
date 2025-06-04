use std::path::Path;

use testcontainers::{runners::AsyncRunner, ContainerAsync, RunnableImage};

use crate::{
    docker_utils::temp_dir_to_mount,
    images::{Varnish, VarnishArgs},
};

pub async fn start_varnish(network: &str, vcl_dir: impl AsRef<Path>) -> ContainerAsync<Varnish> {
    let varnish_image = RunnableImage::from((Varnish::default(), VarnishArgs::new()));
    let vcl_config_volume = temp_dir_to_mount(vcl_dir, "/etc/varnish/");
    varnish_image
        .with_network(network)
        .with_mount(vcl_config_volume)
        .start()
        .await
        .expect("Start varnish container")
}
