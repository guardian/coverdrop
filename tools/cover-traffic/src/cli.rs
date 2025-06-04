use clap::{Parser, Subcommand};

use common::{
    aws::ssm::prefix::ParameterPrefix,
    clap::{AwsConfig, KinesisConfig},
};
use reqwest::Url;

#[derive(Subcommand, Debug)]
pub enum ContinuousTrafficMode {
    /// Create traffic using values defined in parameter store
    ParameterStore {
        #[clap(name = "aws-parameter-prefix", long, env = "AWS_PARAMETER_PREFIX")]
        parameter_prefix: ParameterPrefix,
    },
    /// Creates traffic at a constant rate using provided values
    Manual {
        /// The rate of user-to-journalist messages to send (in messages per hour)
        #[clap(long, env = "MPH_U2J")]
        mph_u2j: u32,
        /// The rate of journalist-to-user messages to send (in messages per hour)
        #[clap(long, env = "MPH_J2U")]
        mph_j2u: u32,
    },
}

#[derive(Subcommand, Debug)]
pub enum TrafficCommand {
    /// Sends the given number of cover traffic messages as fast as possible and then exits.
    Burst {
        /// The number of user-to-journalist messages to send
        #[clap(long)]
        num_u2j: u32,
        /// The number of journalist-to-user messages to send
        #[clap(long)]
        num_j2u: u32,
    },
    /// Sends cover messages continuously at the given rate until the program is manually terminated.
    Continuous {
        #[clap(subcommand)]
        mode: ContinuousTrafficMode,
    },
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct Cli {
    /// The address of the CoverDrop API server
    #[clap(long)]
    #[clap(long, env = "API_URL")]
    pub api_url: Url,
    /// The address of the Fastly service that forwards messages to Kinesis
    #[clap(long, env = "MESSAGING_URL")]
    pub messaging_url: Url,

    /// The mode of operation: burst or continuous; and their parameters
    #[clap(subcommand)]
    pub command: TrafficCommand,

    #[command(flatten)]
    pub aws_config: AwsConfig,

    #[command(flatten)]
    pub kinesis_config: KinesisConfig,
}
