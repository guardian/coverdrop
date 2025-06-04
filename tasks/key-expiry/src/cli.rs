use std::path::PathBuf;

use clap::{Args, Parser};
use common::aws::ssm::prefix::ParameterPrefix;
use reqwest::Url;

#[derive(Args)]
#[group(required = true, multiple = false)]
/// The keys are either fetched from local disk, or from AWS
pub struct TrustedOrgPkLocation {
    /// The path to the directory where the trusted org pk is. This is mutually exclusive with --aws-parameter-prefix
    #[clap(long)]
    pub keys_path: Option<PathBuf>,
    /// The SSM parameter prefix. This is mutually exclusive with --keys-path
    #[clap(long, name = "aws_parameter_prefix", env = "AWS_PARAMETER_PREFIX")]
    pub parameter_prefix: Option<ParameterPrefix>,
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct Cli {
    /// The address of the CoverDrop API server
    #[clap(long, env = "API_URL")]
    pub api_url: Url,
    #[clap(long, default_value = "N/A", env = "TEAM_EMAIL_ADDRESS")]
    pub team_email_address: String,
    #[command(flatten)]
    pub key_location: TrustedOrgPkLocation,
}
