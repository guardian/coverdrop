use std::time::Duration;
use std::{env, net::IpAddr, path::Path};

use crate::{
    constants::{KINESIS_PORT, POSTGRES_PORT},
    containers::start_with_retry::start_with_retry,
    docker_utils::temp_dir_to_mount,
    images::{Api, ApiArgs},
    panic_handler::register_container_panic_hook,
};
use chrono::{DateTime, Utc};
use testcontainers::ContainerAsync;
use testcontainers::{core::Host, ImageExt};

#[allow(clippy::too_many_arguments)]
pub async fn start_api(
    network: &str,
    keys_dir: impl AsRef<Path>,
    postgres_ip: IpAddr,
    base_time: DateTime<Utc>,
    delete_old_dead_drops_poll_seconds: Option<i64>,
    default_journalist_id: Option<String>,
    kinesis_ip: IpAddr,
    minio_url: String,
    minio_host: Host,
) -> ContainerAsync<Api> {
    let api_image_args = ApiArgs::new(
        postgres_ip,
        POSTGRES_PORT,
        base_time,
        delete_old_dead_drops_poll_seconds,
        default_journalist_id,
        kinesis_ip,
        KINESIS_PORT,
        minio_url,
        minio_host.clone(),
    );
    let cmd = api_image_args.into_cmd();

    let keys_volume = temp_dir_to_mount(keys_dir, "/var/keys");

    let api = start_with_retry("API", || {
        Api::default()
            .with_cmd(cmd.clone())
            .with_mount(keys_volume.clone())
            .with_network(network)
            // We want to be able to issue presigned urls from minio on the `localhost` domain,
            // This means we need to able to call minio on the localhost domain from the s3 client in the api
            // This is why we have setup a local hosts entry to map localhost to the minio IP address.
            .with_host("localhost", minio_host.clone())
            .with_startup_timeout(Duration::from_secs(120))
    })
    .await
    .expect("Start container");

    if env::var("PRINT_API_CONTAINER_LOGS").is_ok() {
        register_container_panic_hook("API", api.id());
    }

    api
}
