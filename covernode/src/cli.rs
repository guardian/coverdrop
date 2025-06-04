use std::num::NonZeroU32;
use std::path::PathBuf;

use clap::Parser;
use common::api::models::covernode_id::CoverNodeIdentity;
use common::aws::ssm::prefix::ParameterPrefix;
use common::clap::{AwsConfig, CliSecret, KinesisConfig, PlainRedactor};
use common::task::RunnerMode;
use covernode::DEFAULT_PORT;
use reqwest::Url;

/// The number of seconds to wait between refreshing the journalist tag cache
const JOURNALIST_CACHE_REFRESH_PERIOD_SECONDS: &str = "60";

/// The rate at which the create keys task will run
const CREATE_KEYS_TASK_PERIOD_SECONDS: &str = "60";

/// The rate at which the publish keys task will run
const PUBLISH_KEYS_TASK_PERIOD_SECONDS: &str = "60";

/// The rate at which the delete expired keys task will run
const DELETE_EXPIRED_KEYS_TASK_PERIOD_SECONDS: &str = "60";

// The rate at which the trusted organization public key polling task will run
const ANCHOR_ORGANIZATION_PUBLIC_KEY_POLLING_PERIOD_SECONDS: &str = "60";

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct Cli {
    // The stage the CoverNode is running in
    #[clap(long, value_enum)]
    pub stage: common::clap::Stage,
    /// The identity of this CoverNode
    #[clap(long)]
    pub covernode_id: CoverNodeIdentity,
    /// The path to the CoverNode key database, will be created if it doesn't already exist.
    #[clap(long)]
    pub db_path: PathBuf,

    /// The password for the CoverNode key database.
    #[clap(long)]
    pub db_password: CliSecret<String, PlainRedactor>,
    /// The port that the CoverNodes web server runs on
    #[clap(long, default_value_t = DEFAULT_PORT)]
    pub port: u16,

    /// The base URL of the CoverDrop API
    #[clap(long)]
    pub api_url: Url,
    /// The base URL of the CoverDrop Identity API
    #[clap(long)]
    pub identity_api_url: Url,
    /// A path to the directory where the sequence numbers of the Kinesis checkpoints are stored.
    #[clap(long)]
    pub checkpoint_path: PathBuf,

    #[command(flatten)]
    pub aws_config: AwsConfig,

    /// The SSM parameter prefix needed to fetch the trusted organization public key.
    #[clap(name = "aws-parameter-prefix", long, env = "AWS_PARAMETER_PREFIX")]
    pub parameter_prefix: Option<ParameterPrefix>,

    #[command(flatten)]
    pub kinesis_config: KinesisConfig,

    /// A path to the directory where the various CoverNode keys are.
    #[clap(long)]
    pub keys_path: PathBuf,
    /// Instructs the process to park when the main function exits in an error state
    #[clap(long)]
    pub park_on_error: bool,

    /// Sets the user->journalist input threshold_min
    #[clap(long)]
    pub u2j_threshold_min: usize,
    /// Sets the user->journalist input threshold_max
    #[clap(long)]
    pub u2j_threshold_max: usize,
    /// Sets the user->journalist input timeout duration (in seconds)
    #[clap(long)]
    pub u2j_timeout_seconds: u32,
    /// Sets the user->journalist output batch size
    #[clap(long)]
    pub u2j_output_size: usize,

    /// Sets the journalist->user input threshold_min
    #[clap(long)]
    pub j2u_threshold_min: usize,
    /// Sets the journalist->user input threshold_max
    #[clap(long)]
    pub j2u_threshold_max: usize,
    /// Sets the journalist->user input timeout duration (in seconds)
    #[clap(long)]
    pub j2u_timeout_seconds: u32,
    /// Sets the journalist->user output batch size
    #[clap(long)]
    pub j2u_output_size: usize,

    /// The amount of time to wait between refreshing journalist keys.
    #[clap(long, default_value = JOURNALIST_CACHE_REFRESH_PERIOD_SECONDS)]
    pub journalist_cache_refresh_period_seconds: NonZeroU32,

    /// The amount of time in seconds to wait between attempting to create keys
    #[clap(long, default_value = CREATE_KEYS_TASK_PERIOD_SECONDS)]
    pub create_keys_task_period_seconds: NonZeroU32,

    /// The amount of time in seconds to wait between attempting to publish keys
    #[clap(long, default_value = PUBLISH_KEYS_TASK_PERIOD_SECONDS)]
    pub publish_keys_task_period_seconds: NonZeroU32,

    /// The amount of time in seconds to wait between attempting to delete expired keys
    #[clap(long, default_value = DELETE_EXPIRED_KEYS_TASK_PERIOD_SECONDS)]
    pub delete_expired_keys_task_period_seconds: NonZeroU32,

    /// The amount of time in seconds to wait between polling for new org public keys
    #[clap(long, default_value = ANCHOR_ORGANIZATION_PUBLIC_KEY_POLLING_PERIOD_SECONDS)]
    pub anchor_organization_public_key_polling_period_seconds: NonZeroU32,

    /// The mode to start the task runner for either time based execution or manually triggered
    /// via a web server.
    #[clap(long, default_value = "timer")]
    pub task_runner_mode: RunnerMode,

    /// Disable stream throttling, will cause the CoverNode to breach limitations if done against
    /// production streams
    #[clap(long)]
    pub disable_stream_throttle: bool,
}
