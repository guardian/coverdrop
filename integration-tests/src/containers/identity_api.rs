use std::{env, net::IpAddr, path::Path};

use chrono::{DateTime, Utc};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, RunnableImage};

use crate::{
    constants::API_PORT,
    docker_utils::temp_dir_to_mount,
    images::{IdentityApi, IdentityApiArgs},
    panic_handler::register_container_panic_hook,
};

pub async fn start_identity_api(
    network: &str,
    keys_dir: impl AsRef<Path>,
    api_ip: IpAddr,
    base_time: DateTime<Utc>,
) -> ContainerAsync<IdentityApi> {
    let identity_api_image = IdentityApi::default();
    let identity_api_image_args = IdentityApiArgs::new(api_ip, API_PORT, base_time);

    let keys_volume = temp_dir_to_mount(keys_dir, "/var/keys");

    let identity_api_image = RunnableImage::from((identity_api_image, identity_api_image_args));

    let api = identity_api_image
        .with_mount(keys_volume)
        .with_network(network)
        .start()
        .await
        .expect("Start identity api container");

    if env::var("PRINT_IDENTITY_API_CONTAINER_LOGS").is_ok() {
        register_container_panic_hook("IDENTITY_API", api.id());
    }

    api
}
