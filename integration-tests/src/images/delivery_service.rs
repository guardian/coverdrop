use std::{borrow::Cow, collections::HashMap, env, net::IpAddr};

use chrono::{DateTime, Utc};
use testcontainers::{core::WaitFor, Image};

use crate::{
    constants::{POSTGRES_DB, POSTGRES_PASSWORD, POSTGRES_USER},
    docker_utils::date_time_to_set_faketime_command_string,
};

#[derive(Debug, Clone)]
pub struct DeliveryServiceArgs {
    api_ip: IpAddr,
    api_port: u16,
    db_ip: IpAddr,
    db_port: u16,
    base_time: DateTime<Utc>,
}

impl DeliveryServiceArgs {
    pub fn new(
        api_ip: IpAddr,
        api_port: u16,
        db_ip: IpAddr,
        db_port: u16,
        base_time: DateTime<Utc>,
    ) -> Self {
        Self {
            api_ip,
            api_port,
            db_ip,
            db_port,
            base_time,
        }
    }
}

impl DeliveryServiceArgs {
    pub fn into_cmd(self) -> Vec<String> {
        let set_time_arg = date_time_to_set_faketime_command_string(self.base_time);

        let api_url_arg = format!("--api-url=http://{}:{}", self.api_ip, self.api_port);

        let postgres_arg = format!(
            "--db-url=postgresql://{}:{}@{}:{}/{}",
            POSTGRES_USER, POSTGRES_PASSWORD, self.db_ip, self.db_port, POSTGRES_DB
        );

        let command = format!(
            "{set_time_arg} && ./delivery-service --stage=dev {api_url_arg} {postgres_arg}"
        );

        println!("Starting Delivery Service with: {command}");

        vec!["/bin/bash".into(), "-c".into(), command]
    }
}

#[derive(Debug)]
pub struct DeliveryService {
    name: String,
    tag: String,
    env_vars: HashMap<String, String>,
}

impl Default for DeliveryService {
    fn default() -> Self {
        let mut env_vars = HashMap::new();
        env_vars.insert("RUST_LOG".into(), "DEBUG".into());
        env_vars.insert("FAKETIME_TIMESTAMP_FILE".into(), "/faketime".into());
        Self {
            name: env::var("DELIVERY_SERVICE_IMAGE_NAME")
                .unwrap_or("test_coverdrop_delivery_service".into()),
            tag: env::var("DELIVERY_SERVICE_IMAGE_TAG").unwrap_or("dev".into()),
            env_vars,
        }
    }
}

impl Image for DeliveryService {
    fn name(&self) -> &str {
        &self.name
    }

    fn tag(&self) -> &str {
        &self.tag
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::message_on_stdout("Starting server on")]
    }

    fn env_vars(
        &self,
    ) -> impl IntoIterator<Item = (impl Into<Cow<'_, str>>, impl Into<Cow<'_, str>>)> {
        self.env_vars.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }
}
