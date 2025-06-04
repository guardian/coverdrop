use std::{env, net::IpAddr, path::Path};

use crate::images::{dev_j2u_mixing_config, dev_u2j_mixing_config};
use crate::{
    constants::{IDENTITY_API_PORT, KINESIS_PORT},
    docker_utils::temp_dir_to_mount,
    images::{CoverNode, CoverNodeArgs},
    panic_handler::register_container_panic_hook,
};
use chrono::{DateTime, Utc};
use common::api::models::covernode_id::CoverNodeIdentity;
use common::task::RunnerMode;
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, RunnableImage};

const CONTAINER_KEYS_DIR: &str = "/var/keys";
const CONTAINER_CHECKPOINTS_DIR: &str = "/var/checkpoints";

#[allow(clippy::too_many_arguments)]
pub async fn start_covernode(
    covernode_id: CoverNodeIdentity,
    network: &str,
    keys_path: impl AsRef<Path>,
    checkpoints_path: impl AsRef<Path>,
    api_ip: IpAddr,
    api_port: u16,
    identity_api_ip: IpAddr,
    kinesis_ip: IpAddr,
    base_time: DateTime<Utc>,
    runner_mode: RunnerMode,
) -> ContainerAsync<CoverNode> {
    let keys_volume = temp_dir_to_mount(keys_path, CONTAINER_KEYS_DIR);
    let checkpoints_volume = temp_dir_to_mount(checkpoints_path, CONTAINER_CHECKPOINTS_DIR);

    let covernode_image = CoverNode::default();
    let covernode_image_args = CoverNodeArgs::new(
        covernode_id,
        api_ip,
        api_port,
        identity_api_ip,
        IDENTITY_API_PORT,
        CONTAINER_CHECKPOINTS_DIR.into(),
        CONTAINER_KEYS_DIR.into(),
        kinesis_ip,
        KINESIS_PORT,
        base_time,
        dev_u2j_mixing_config(),
        dev_j2u_mixing_config(),
        runner_mode,
    );

    let covernode_image = RunnableImage::from((covernode_image, covernode_image_args));

    let covernode = covernode_image
        .with_mount(keys_volume)
        .with_mount(checkpoints_volume)
        .with_network(network)
        .start()
        .await;

    let covernode = covernode.expect("Start covernode container");

    if env::var("PRINT_COVERNODE_CONTAINER_LOGS").is_ok() {
        register_container_panic_hook("CoverNode", covernode.id());
    }

    covernode
}
