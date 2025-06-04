use std::{num::NonZeroU32, path::PathBuf};

use common::{
    api::models::journalist_id::JournalistIdentity,
    aws::ssm::prefix::ParameterPrefix,
    clap::{AwsConfig, CliSecret, KinesisConfig, PostgresConnectionStringRedactor},
    task::RunnerMode,
};

use clap::{Args, Parser};

// 360 J2U dead drops is about 4MB.
// This is the equivalent of 15 days of dead drops at 1 per hour.
const J2U_DEAD_DROP_LIMIT: &str = "360";

// 24 U2J dead drops should be roughly 24MB, one day of dead drops
const U2J_DEAD_DROP_LIMIT: &str = "24";

#[derive(Debug, Args, Clone)]
#[group(required = true, multiple = false)]
/// The keys are either fetched from local disk, or from AWS
pub struct KeyLocation {
    /// The path to the directory where the various keys are. This is mutually exclusive with --aws-parameter-prefix
    #[clap(long, global = true)]
    pub keys_path: Option<PathBuf>,
    /// The SSM parameter prefix. This is mutually exclusive with --keys-path
    #[clap(long, name = "aws_parameter_prefix", env = "AWS_PARAMETER_PREFIX")]
    pub parameter_prefix: Option<ParameterPrefix>,
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct Cli {
    #[clap(long)]
    pub stage: common::clap::Stage,
    /// The path to the Postgres database. If not set, the application will try to
    /// read it from the COVERDROP_API_DB_URL environment variable
    #[clap(long, env = "COVERDROP_API_DB_URL")]
    pub db_url: CliSecret<String, PostgresConnectionStringRedactor>,
    /// The ID for the default journalist.
    #[clap(long)]
    pub default_journalist_id: Option<JournalistIdentity>,
    /// The amount of time in seconds to wait between polling for dead drops to remove
    /// Must be more than 1.
    #[clap(long)]
    pub delete_old_dead_drops_polling_period_seconds: Option<i64>,
    #[clap(long)]
    /// The maximum amount of database connections that pool can maintain.
    /// If not set, it will default to the sqlx default, which is 10.
    #[clap(long, env = "MAX_DB_CONNECTIONS")]
    pub max_db_connections: Option<u32>,
    /// The amount of time in seconds to wait between polling for new org public keys
    /// Must be more than 1.
    #[clap(long)]
    pub anchor_organization_public_key_polling_period_seconds: Option<i64>,

    #[command(flatten)]
    pub key_location: KeyLocation,

    #[command(flatten)]
    pub kinesis_config: KinesisConfig,

    #[command(flatten)]
    pub aws_config: AwsConfig,

    /// The maximum number of dead drops to return per request on the user to journalist endpoint
    #[clap(long, default_value = U2J_DEAD_DROP_LIMIT)]
    pub u2j_dead_drops_per_request_limit: NonZeroU32,

    /// The maximum number of dead drops to return per request on the journalist to user endpoint
    #[clap(long, default_value = J2U_DEAD_DROP_LIMIT)]
    pub j2u_dead_drops_per_request_limit: NonZeroU32,

    /// The mode to start the task runner for either time based execution or manually triggered
    /// via a web server.
    #[clap(long, default_value = "timer")]
    pub task_runner_mode: RunnerMode,
}
