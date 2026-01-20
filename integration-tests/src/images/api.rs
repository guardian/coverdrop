use std::{collections::HashMap, env, net::IpAddr};

use chrono::{DateTime, Utc};
use testcontainers::{
    core::{Host, WaitFor},
    Image, ImageArgs,
};

use crate::{
    constants::{POSTGRES_DB, POSTGRES_PASSWORD, POSTGRES_USER},
    docker_utils::date_time_to_set_faketime_command_string,
    secrets::{API_AWS_ACCESS_KEY_ID_SECRET, API_AWS_SECRET_ACCESS_KEY_SECRET},
};

#[derive(Debug, Clone)]
pub struct ApiArgs {
    db_ip: IpAddr,
    db_port: u16,
    base_time: DateTime<Utc>,
    delete_old_dead_drops_poll_seconds: Option<i64>,
    default_journalist_id: Option<String>,
    kinesis_ip: IpAddr,
    kinesis_port: u16,
    minio_url: String,
    #[allow(dead_code)]
    minio_host: Host,
}

impl ApiArgs {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        db_ip: IpAddr,
        db_port: u16,
        base_time: DateTime<Utc>,
        delete_old_dead_drops_poll_seconds: Option<i64>,
        default_journalist_id: Option<String>,
        kinesis_ip: IpAddr,
        kinesis_port: u16,
        minio_url: String,
        minio_host: Host,
    ) -> Self {
        Self {
            db_ip,
            db_port,
            base_time,
            delete_old_dead_drops_poll_seconds,
            default_journalist_id,
            kinesis_ip,
            kinesis_port,
            minio_url,
            minio_host,
        }
    }
}

impl ImageArgs for ApiArgs {
    fn into_iterator(self) -> Box<dyn Iterator<Item = String>> {
        let set_time_arg = date_time_to_set_faketime_command_string(self.base_time);

        let postgres_arg = format!(
            "--db-url=postgres://{}:{}@{}:{}/{}",
            POSTGRES_USER, POSTGRES_PASSWORD, self.db_ip, self.db_port, POSTGRES_DB
        );

        let delete_old_dead_drops_poll_seconds_arg = self
            .delete_old_dead_drops_poll_seconds
            .map(|seconds| format!("--delete-old-dead-drops-polling-period-seconds={seconds}"))
            .unwrap_or_else(|| "".to_owned());

        let default_journalist_id_arg = self
            .default_journalist_id
            .map(|id| format!("--default-journalist-id={id}"))
            .unwrap_or_else(|| "".to_owned());

        let kinesis_flags = format!(
            "--kinesis-endpoint=http://{}:{} --kinesis-journalist-stream=journalist-messages --kinesis-user-stream=user-messages ",
            self.kinesis_ip, self.kinesis_port
        );

        let minio_flags = format!("--s3-endpoint-url={}", self.minio_url);

        let task_runner_mode = "--task-runner-mode=timer-and-manually-triggered";
        let command = format!(
            "{set_time_arg} && ./api --stage=dev --keys-path=/var/keys {postgres_arg} \
            {delete_old_dead_drops_poll_seconds_arg} {default_journalist_id_arg} \
            {task_runner_mode} {kinesis_flags} {minio_flags}"
        );

        println!("Starting API with: {command}");

        Box::new(vec!["/bin/bash".into(), "-c".into(), command].into_iter())
    }
}

#[derive(Debug)]
pub struct Api {
    env_vars: HashMap<String, String>,
}

impl Default for Api {
    fn default() -> Self {
        let mut env_vars = HashMap::new();

        env_vars.insert("STAGE".into(), "dev".into());
        env_vars.insert("RUST_LOG".into(), "DEBUG".into());
        env_vars.insert("FAKETIME_TIMESTAMP_FILE".into(), "/faketime".into());
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

impl Image for Api {
    type Args = ApiArgs;

    fn name(&self) -> String {
        env::var("API_IMAGE_NAME").unwrap_or("test_coverdrop_api".into())
    }

    fn tag(&self) -> String {
        env::var("API_IMAGE_TAG").unwrap_or("dev".into())
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::message_on_stdout("Starting server on")]
    }

    fn env_vars(&self) -> Box<dyn Iterator<Item = (&String, &String)> + '_> {
        Box::new(self.env_vars.iter())
    }
}
