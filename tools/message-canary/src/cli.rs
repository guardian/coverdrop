use clap::Parser;
use common::clap::{CliSecret, PostgresConnectionStringRedactor, Stage};
use reqwest::Url;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct Cli {
    #[clap(long, value_enum)]
    pub stage: Stage,

    #[clap(long)]
    pub db_url: CliSecret<String, PostgresConnectionStringRedactor>,

    /// The address of the CoverDrop API server
    #[clap(long)]
    pub api_url: Url,
    /// The address of the U2J appender service that forwards messages to Kinesis
    #[clap(long)]
    pub messaging_url: Url,

    /// The path to the directory containing the canary vaults
    #[clap(long)]
    pub vaults_path: PathBuf,

    /// How many users to send traffic
    #[clap(long, default_value_t = 1)]
    pub num_users: u16,

    /// The rate of user-to-journalist messages to send (in messages per hour)
    #[clap(long)]
    pub mph_u2j: u32,
    ///
    /// The rate of journalist-to-user messages to send (in messages per hour)
    #[clap(long)]
    pub mph_j2u: u32,

    /// The maximum number of hours we expect our messages to take
    /// in both directions.
    ///
    /// Messages that take longer to arrive than this duration will be
    /// considered undelivered.
    #[clap(long)]
    pub max_delivery_time_hours: u64,
}
