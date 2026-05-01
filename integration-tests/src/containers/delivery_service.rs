use std::{env, net::IpAddr};

use chrono::{DateTime, Utc};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, ImageExt};

use crate::images::{DeliveryService, DeliveryServiceArgs};
use crate::{constants::POSTGRES_PORT, panic_handler::register_container_panic_hook};

pub async fn start_delivery_service(
    network: &str,
    api_ip: IpAddr,
    api_port: u16,
    postgres_ip: IpAddr,
    base_time: DateTime<Utc>,
) -> ContainerAsync<DeliveryService> {
    let image = DeliveryService::default();
    let args = DeliveryServiceArgs::new(api_ip, api_port, postgres_ip, POSTGRES_PORT, base_time);

    let delivery_service = image
        .with_cmd(args.into_cmd())
        .with_network(network)
        .start()
        .await
        .expect("Start Delivery Service container");

    if env::var("PRINT_DELIVERY_SERVICE_CONTAINER_LOGS").is_ok() {
        register_container_panic_hook("Delivery Service", delivery_service.id());
    }

    delivery_service
}
