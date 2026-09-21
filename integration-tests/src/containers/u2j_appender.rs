use std::time::Duration;
use std::{env, net::IpAddr};
use testcontainers::{ContainerAsync, ImageExt};

use crate::containers::start_with_retry::start_with_retry;
use crate::images::{U2JAppender, U2JAppenderArgs};
use crate::{constants::KINESIS_PORT, panic_handler::register_container_panic_hook};

pub async fn start_u2j_appender(network: &str, kinesis_ip: IpAddr) -> ContainerAsync<U2JAppender> {
    let args = U2JAppenderArgs::new(kinesis_ip, KINESIS_PORT);
    let cmd = args.into_cmd();

    let u2j_appender = start_with_retry("U2J Appender", || {
        U2JAppender::default()
            .with_cmd(cmd.clone())
            .with_network(network)
            .with_startup_timeout(Duration::from_secs(60))
    })
    .await
    .expect("Start U2J Appender container");

    if env::var("PRINT_U2J_APPENDER_CONTAINER_LOGS").is_ok() {
        register_container_panic_hook("U2J Appender", u2j_appender.id());
    }

    u2j_appender
}
