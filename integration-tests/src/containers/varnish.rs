use std::path::Path;
use std::time::Duration;

use testcontainers::{ContainerAsync, ImageExt};

use crate::{
    containers::start_with_retry::start_with_retry,
    docker_utils::temp_dir_to_mount,
    images::{Varnish, VarnishArgs},
};

pub async fn start_varnish(network: &str, vcl_dir: impl AsRef<Path>) -> ContainerAsync<Varnish> {
    let vcl_config_volume = temp_dir_to_mount(vcl_dir, "/etc/varnish/");
    start_with_retry("Varnish", || {
        Varnish::default()
            .with_cmd(VarnishArgs::new().into_cmd())
            .with_network(network)
            .with_mount(vcl_config_volume.clone())
            .with_startup_timeout(Duration::from_secs(60))
    })
    .await
    .expect("Start varnish container")
}
