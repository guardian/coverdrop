use std::{collections::HashMap, env, net::IpAddr};

use testcontainers::{core::WaitFor, Image, ImageArgs};

use crate::secrets::{API_AWS_ACCESS_KEY_ID_SECRET, API_AWS_SECRET_ACCESS_KEY_SECRET};

#[derive(Debug, Clone)]
pub struct U2JAppenderArgs {
    kinesis_ip: IpAddr,
    kinesis_port: u16,
}

impl U2JAppenderArgs {
    pub fn new(kinesis_ip: IpAddr, kinesis_port: u16) -> Self {
        Self {
            kinesis_ip,
            kinesis_port,
        }
    }
}

impl ImageArgs for U2JAppenderArgs {
    fn into_iterator(self) -> Box<dyn Iterator<Item = String>> {
        let kinesis_flags = format!(
            "--kinesis-endpoint=http://{}:{} --kinesis-u2j-stream=user-messages ",
            self.kinesis_ip, self.kinesis_port
        );

        let command = format!("./u2j-appender --stage=dev {kinesis_flags}");

        Box::new(vec!["/bin/bash".into(), "-c".into(), command].into_iter())
    }
}

#[derive(Debug)]
pub struct U2JAppender {
    env_vars: HashMap<String, String>,
}

impl Default for U2JAppender {
    fn default() -> Self {
        let mut env_vars = HashMap::new();

        env_vars.insert("RUST_LOG".into(), "DEBUG".into());
        env_vars.insert(
            "AWS_ACCESS_KEY_ID".into(),
            API_AWS_ACCESS_KEY_ID_SECRET.into(),
        );
        env_vars.insert(
            "AWS_SECRET_ACCESS_KEY".into(),
            API_AWS_SECRET_ACCESS_KEY_SECRET.into(),
        );

        Self { env_vars }
    }
}

impl Image for U2JAppender {
    type Args = U2JAppenderArgs;

    fn name(&self) -> String {
        env::var("U2J_APPENDER_IMAGE_NAME").unwrap_or("test_coverdrop_u2j-appender".into())
    }

    fn tag(&self) -> String {
        env::var("U2J_APPENDER_IMAGE_TAG").unwrap_or("dev".into())
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![
            WaitFor::message_on_stdout("Starting server on"),
            // TODO wait for health check
            // Wait.ForUnixContainer().UntilContainerIsHealthy(),
        ]
    }

    fn env_vars(&self) -> Box<dyn Iterator<Item = (&String, &String)> + '_> {
        Box::new(self.env_vars.iter())
    }
}
