use std::{env, net::IpAddr};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, ImageExt};

use crate::images::{U2JAppender, U2JAppenderArgs};
use crate::{constants::KINESIS_PORT, panic_handler::register_container_panic_hook};

pub async fn start_u2j_appender(network: &str, kinesis_ip: IpAddr) -> ContainerAsync<U2JAppender> {
    let image = U2JAppender::default();
    let args = U2JAppenderArgs::new(kinesis_ip, KINESIS_PORT);

    let u2j_appender = image
        .with_cmd(args.into_cmd())
        .with_network(network)
        .start()
        .await
        .expect("Start U2J Appender container");

    if env::var("PRINT_U2J_APPENDER_CONTAINER_LOGS").is_ok() {
        register_container_panic_hook("U2J Appender", u2j_appender.id());
    }

    u2j_appender
}
