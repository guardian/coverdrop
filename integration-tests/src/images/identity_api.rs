use std::{collections::HashMap, env, net::IpAddr};

use chrono::{DateTime, Utc};
use common::task::RunnerMode;
use testcontainers::{core::WaitFor, Image, ImageArgs};

use crate::{
    constants::IDENTITY_API_DB_PASSWORD,
    docker_utils::date_time_to_set_faketime_command_string,
    secrets::{IDENTITY_API_AWS_ACCESS_KEY_ID_SECRET, IDENTITY_API_AWS_SECRET_ACCESS_KEY_SECRET},
};

#[derive(Debug, Clone)]
pub struct IdentityApiArgs {
    api_ip: IpAddr,
    api_port: u16,
    runner_mode: RunnerMode,
    base_time: DateTime<Utc>,
}

impl IdentityApiArgs {
    pub fn new(
        api_ip: IpAddr,
        api_port: u16,
        runner_mode: RunnerMode,
        base_time: DateTime<Utc>,
    ) -> Self {
        Self {
            api_ip,
            api_port,
            runner_mode,
            base_time,
        }
    }
}

impl ImageArgs for IdentityApiArgs {
    fn into_iterator(self) -> Box<dyn Iterator<Item = String>> {
        let set_time_arg = date_time_to_set_faketime_command_string(self.base_time);

        let api_url_arg = format!("--api-url=http://{}:{}", self.api_ip, self.api_port);

        let db_url_arg = "--db-path=/var/keys/identity-api.db";
        let db_password_arg = format!("--db-password={IDENTITY_API_DB_PASSWORD}",);
        let runner_mode_arg = format!("--task-runner-mode={}", self.runner_mode);

        let command =
            format!("{set_time_arg} && ./identity-api --stage=dev --keys-path=/var/keys {runner_mode_arg} {api_url_arg} {db_url_arg} {db_password_arg}");

        println!("Starting Identity API with: {command}");

        Box::new(vec!["/bin/bash".into(), "-c".into(), command].into_iter())
    }
}

#[derive(Debug)]
pub struct IdentityApi {
    env_vars: HashMap<String, String>,
}

impl Default for IdentityApi {
    fn default() -> Self {
        let mut env_vars = HashMap::new();

        env_vars.insert("RUST_LOG".into(), "DEBUG".into());
        env_vars.insert("FAKETIME_TIMESTAMP_FILE".into(), "/faketime".into());
        env_vars.insert(
            "AWS_ACCESS_KEY_ID".into(),
            IDENTITY_API_AWS_ACCESS_KEY_ID_SECRET.into(),
        );
        env_vars.insert(
            "AWS_SECRET_ACCESS_KEY".into(),
            IDENTITY_API_AWS_SECRET_ACCESS_KEY_SECRET.into(),
        );

        Self { env_vars }
    }
}

impl Image for IdentityApi {
    type Args = IdentityApiArgs;

    fn name(&self) -> String {
        env::var("IAPI_IMAGE_NAME").unwrap_or("test_coverdrop_identity-api".into())
    }

    fn tag(&self) -> String {
        env::var("IAPI_IMAGE_TAG").unwrap_or("dev".into())
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::message_on_stdout("Starting identity API server")]
    }

    fn env_vars(&self) -> Box<dyn Iterator<Item = (&String, &String)> + '_> {
        Box::new(self.env_vars.iter())
    }
}
