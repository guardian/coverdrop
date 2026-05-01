use std::path::Path;

use testcontainers::{runners::AsyncRunner, ContainerAsync, ImageExt};

use crate::{
    docker_utils::temp_dir_to_mount,
    images::{Varnish, VarnishArgs},
};

pub async fn start_varnish(network: &str, vcl_dir: impl AsRef<Path>) -> ContainerAsync<Varnish> {
    let vcl_config_volume = temp_dir_to_mount(vcl_dir, "/etc/varnish/");
    Varnish::default()
        .with_cmd(VarnishArgs::new().into_cmd())
        .with_network(network)
        .with_mount(vcl_config_volume)
        .start()
        .await
        .expect("Start varnish container")
}
