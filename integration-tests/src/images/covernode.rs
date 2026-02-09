use chrono::{DateTime, Duration, Utc};
use common::api::models::covernode_id::CoverNodeIdentity;
use common::task::RunnerMode;
use covernode_app::mixing::mixing_strategy::MixingStrategyConfiguration;
use std::{collections::HashMap, env, net::IpAddr};
use testcontainers::{core::WaitFor, Image, ImageArgs};

use crate::constants::COVERNODE_DB_PASSWORD;
use crate::{
    docker_utils::date_time_to_set_faketime_command_string,
    secrets::{COVERNODE_AWS_ACCESS_KEY_ID_SECRET, COVERNODE_AWS_SECRET_ACCESS_KEY_SECRET},
};

#[derive(Debug, Clone)]
pub struct CoverNodeArgs {
    covernode_id: CoverNodeIdentity,
    api_ip: IpAddr,
    api_port: u16,
    identity_api_ip: IpAddr,
    identity_api_port: u16,
    checkpoint_dir: String,
    keys_dir: String,
    kinesis_ip: IpAddr,
    kinesis_port: u16,
    base_time: DateTime<Utc>,
    u2j_mixing_config: MixingStrategyConfiguration,
    j2u_mixing_config: MixingStrategyConfiguration,
    runner_mode: RunnerMode,
}

/** See: docs/covernode_mixing.md */
pub fn dev_u2j_mixing_config() -> MixingStrategyConfiguration {
    MixingStrategyConfiguration::new(2, 10, "U2JMixerLevel", Duration::seconds(900), 10)
}

/** See: docs/covernode_mixing.md */
pub fn dev_j2u_mixing_config() -> MixingStrategyConfiguration {
    MixingStrategyConfiguration::new(10, 40, "J2UMixerLevel", Duration::seconds(900), 5)
}

impl CoverNodeArgs {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        covernode_id: CoverNodeIdentity,
        api_ip: IpAddr,
        api_port: u16,
        identity_api_ip: IpAddr,
        identity_api_port: u16,
        checkpoint_dir: String,
        keys_dir: String,
        kinesis_ip: IpAddr,
        kinesis_port: u16,
        base_time: DateTime<Utc>,
        u2j_mixing_config: MixingStrategyConfiguration,
        j2u_mixing_config: MixingStrategyConfiguration,
        runner_mode: RunnerMode,
    ) -> Self {
        Self {
            covernode_id,
            api_ip,
            api_port,
            identity_api_ip,
            identity_api_port,
            checkpoint_dir,
            keys_dir,
            kinesis_ip,
            kinesis_port,
            base_time,
            u2j_mixing_config,
            j2u_mixing_config,
            runner_mode,
        }
    }
}

impl ImageArgs for CoverNodeArgs {
    fn into_iterator(self) -> Box<dyn Iterator<Item = String>> {
        let set_time_arg = date_time_to_set_faketime_command_string(self.base_time);

        let covernode_id = self.covernode_id;

        let db_args = format!(
            "--db-path={}/{}.db --db-password={}",
            self.keys_dir, covernode_id, COVERNODE_DB_PASSWORD
        );

        let api_url_arg = format!("--api-url=http://{}:{}", self.api_ip, self.api_port);
        let identity_api_url_arg = format!(
            "--identity-api-url=http://{}:{}",
            self.identity_api_ip, self.identity_api_port
        );

        let checkpoint_dir_arg = format!("--checkpoint-path={}", self.checkpoint_dir);

        let journalist_cache_refresh_period_seconds = "--journalist-cache-refresh-period-seconds=1";

        let publish_keys_task_period_seconds_args = "--publish-keys-task-period-seconds=1";

        let kinesis_flags = format!(
            "--kinesis-endpoint=http://{}:{} --kinesis-user-stream=user-messages --kinesis-journalist-stream=journalist-messages",
            self.kinesis_ip,
            self.kinesis_port
        );
        let aws_flags = "--aws-region=eu-west-1";

        let u2j_mixing_parameters = format!(
            "--u2j-threshold-min={} --u2j-threshold-max={} --u2j-timeout-seconds={} --u2j-output-size={}",
            self.u2j_mixing_config.threshold_min,
            self.u2j_mixing_config.threshold_max,
            self.u2j_mixing_config.timeout.num_seconds(),
            self.u2j_mixing_config.output_size
        );

        let j2u_mixing_parameters = format!(
            "--j2u-threshold-min={} --j2u-threshold-max={} --j2u-timeout-seconds={} --j2u-output-size={}",
            self.j2u_mixing_config.threshold_min,
            self.j2u_mixing_config.threshold_max,
            self.j2u_mixing_config.timeout.num_seconds(),
            self.j2u_mixing_config.output_size
        );

        let runner_mode_arg = format!("--task-runner-mode={}", self.runner_mode);

        let stage = "--stage=dev";

        let disable_stream_throttle = "--disable-stream-throttle";

        let command = format!(
            "{set_time_arg} && ./covernode --covernode-id {covernode_id} {db_args} \
            {api_url_arg} {identity_api_url_arg} {checkpoint_dir_arg} {journalist_cache_refresh_period_seconds} \
            {publish_keys_task_period_seconds_args} \
            {kinesis_flags} {u2j_mixing_parameters} \
            {j2u_mixing_parameters} {aws_flags} {runner_mode_arg} \
            {disable_stream_throttle} {stage}");

        println!("Starting Covernode with: {command}");

        Box::new(vec!["/bin/bash".into(), "-c".into(), command].into_iter())
    }
}

#[derive(Debug)]
pub struct CoverNode {
    env_vars: HashMap<String, String>,
}

impl Default for CoverNode {
    fn default() -> Self {
        let mut env_vars = HashMap::new();

        env_vars.insert("FAKETIME_TIMESTAMP_FILE".into(), "/faketime".into());

        env_vars.insert("AWS_REGION".into(), "eu-west-1".into());

        env_vars.insert(
            "AWS_ACCESS_KEY_ID".into(),
            COVERNODE_AWS_ACCESS_KEY_ID_SECRET.into(),
        );
        env_vars.insert(
            "AWS_SECRET_ACCESS_KEY".into(),
            COVERNODE_AWS_SECRET_ACCESS_KEY_SECRET.into(),
        );

        Self { env_vars }
    }
}

impl Image for CoverNode {
    type Args = CoverNodeArgs;

    fn name(&self) -> String {
        env::var("COVERNODE_IMAGE_NAME").unwrap_or("test_coverdrop_covernode".into())
    }

    fn tag(&self) -> String {
        env::var("COVERNODE_IMAGE_TAG").unwrap_or("dev".into())
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::message_on_stdout(
            "Started CoverNode service journalist->user",
        )]
    }

    fn env_vars(&self) -> Box<dyn Iterator<Item = (&String, &String)> + '_> {
        Box::new(self.env_vars.iter())
    }
}
