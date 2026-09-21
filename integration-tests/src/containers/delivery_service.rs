use std::time::Duration;
use std::{env, net::IpAddr};

use chrono::{DateTime, Utc};
use testcontainers::{ContainerAsync, ImageExt};

use crate::containers::start_with_retry::start_with_retry;
use crate::images::{DeliveryService, DeliveryServiceArgs};
use crate::{constants::POSTGRES_PORT, panic_handler::register_container_panic_hook};

pub async fn start_delivery_service(
    network: &str,
    api_ip: IpAddr,
    api_port: u16,
    postgres_ip: IpAddr,
    base_time: DateTime<Utc>,
) -> ContainerAsync<DeliveryService> {
    let args = DeliveryServiceArgs::new(api_ip, api_port, postgres_ip, POSTGRES_PORT, base_time);
    let cmd = args.into_cmd();

    let delivery_service = start_with_retry("Delivery Service", || {
        DeliveryService::default()
            .with_cmd(cmd.clone())
            .with_network(network)
            .with_startup_timeout(Duration::from_secs(120))
    })
    .await
    .expect("Start Delivery Service container");

    if env::var("PRINT_DELIVERY_SERVICE_CONTAINER_LOGS").is_ok() {
        register_container_panic_hook("Delivery Service", delivery_service.id());
    }

    delivery_service
}
