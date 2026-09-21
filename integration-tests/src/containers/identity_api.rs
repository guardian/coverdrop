use std::{env, net::IpAddr, path::Path, time::Duration};

use chrono::{DateTime, Utc};
use common::task::RunnerMode;
use testcontainers::{ContainerAsync, ImageExt};

use crate::{
    constants::API_PORT,
    containers::start_with_retry::start_with_retry,
    docker_utils::temp_dir_to_mount,
    images::{IdentityApi, IdentityApiArgs},
    panic_handler::register_container_panic_hook,
};

pub async fn start_identity_api(
    network: &str,
    keys_dir: impl AsRef<Path>,
    api_ip: IpAddr,
    runner_mode: RunnerMode,
    base_time: DateTime<Utc>,
) -> ContainerAsync<IdentityApi> {
    let identity_api_image_args = IdentityApiArgs::new(api_ip, API_PORT, runner_mode, base_time);
    let cmd = identity_api_image_args.into_cmd();

    let keys_volume = temp_dir_to_mount(keys_dir, "/var/keys");

    if runner_mode.triggerable() {
        env::set_var("TASK_RUNNER_TRIGGERABLE", "true");
    }

    let api = start_with_retry("Identity API", || {
        IdentityApi::default()
            .with_cmd(cmd.clone())
            .with_mount(keys_volume.clone())
            .with_network(network)
            .with_startup_timeout(Duration::from_secs(120))
    })
    .await
    .expect("Start identity api container");

    if env::var("PRINT_IDENTITY_API_CONTAINER_LOGS").is_ok() {
        register_container_panic_hook("IDENTITY_API", api.id());
    }

    api
}
