use std::{borrow::Cow, collections::HashMap, env, net::IpAddr};

use testcontainers::{core::WaitFor, Image};

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

impl U2JAppenderArgs {
    pub fn into_cmd(self) -> Vec<String> {
        let kinesis_flags = format!(
            "--kinesis-endpoint=http://{}:{} --kinesis-u2j-stream=user-messages ",
            self.kinesis_ip, self.kinesis_port
        );

        let command = format!("./u2j-appender --stage=dev {kinesis_flags}");

        vec!["/bin/bash".into(), "-c".into(), command]
    }
}

#[derive(Debug)]
pub struct U2JAppender {
    name: String,
    tag: String,
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

        Self {
            name: env::var("U2J_APPENDER_IMAGE_NAME")
                .unwrap_or("test_coverdrop_u2j-appender".into()),
            tag: env::var("U2J_APPENDER_IMAGE_TAG").unwrap_or("dev".into()),
            env_vars,
        }
    }
}

impl Image for U2JAppender {
    fn name(&self) -> &str {
        &self.name
    }

    fn tag(&self) -> &str {
        &self.tag
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![
            WaitFor::message_on_stdout("Starting server on"),
            // TODO wait for health check
            // Wait.ForUnixContainer().UntilContainerIsHealthy(),
        ]
    }

    fn env_vars(
        &self,
    ) -> impl IntoIterator<Item = (impl Into<Cow<'_, str>>, impl Into<Cow<'_, str>>)> {
        self.env_vars.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }
}
