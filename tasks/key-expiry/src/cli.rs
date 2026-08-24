use clap::Parser;
use common::{aws::ssm::prefix::ParameterPrefix, clap::Stage};
use reqwest::Url;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct Cli {
    /// The address of the CoverDrop API server
    #[clap(long, env = "API_URL")]
    pub api_url: Url,
    #[clap(long, default_value = "N/A", env = "TEAM_EMAIL_ADDRESS")]
    pub team_email_address: String,
    /// The SSM parameter prefix. This is mutually exclusive with --keys-path
    #[clap(long, name = "aws_parameter_prefix", env = "AWS_PARAMETER_PREFIX")]
    pub parameter_prefix: Option<ParameterPrefix>,
    #[clap(long, env = "STAGE")]
    pub stage: Stage,
}
