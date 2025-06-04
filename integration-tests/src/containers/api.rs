use std::{env, net::IpAddr, path::Path};

use crate::{
    constants::{KINESIS_PORT, POSTGRES_PORT},
    docker_utils::temp_dir_to_mount,
    images::{Api, ApiArgs},
    panic_handler::register_container_panic_hook,
};
use chrono::{DateTime, Utc};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, RunnableImage};

#[allow(clippy::too_many_arguments)]
pub async fn start_api(
    network: &str,
    keys_dir: impl AsRef<Path>,
    postgres_ip: IpAddr,
    base_time: DateTime<Utc>,
    delete_old_dead_drops_poll_seconds: Option<i64>,
    default_journalist_id: Option<String>,
    dead_drop_limit: Option<i64>,
    kinesis_ip: IpAddr,
) -> ContainerAsync<Api> {
    let api_image = Api::default();
    let api_image_args = ApiArgs::new(
        postgres_ip,
        POSTGRES_PORT,
        base_time,
        delete_old_dead_drops_poll_seconds,
        default_journalist_id,
        dead_drop_limit,
        kinesis_ip,
        KINESIS_PORT,
    );

    let keys_volume = temp_dir_to_mount(keys_dir, "/var/keys");

    let api_image = RunnableImage::from((api_image, api_image_args));

    let api = api_image
        .with_mount(keys_volume)
        .with_network(network)
        .start()
        .await
        .expect("Start container");

    if env::var("PRINT_API_CONTAINER_LOGS").is_ok() {
        register_container_panic_hook("API", api.id());
    }

    api
}
