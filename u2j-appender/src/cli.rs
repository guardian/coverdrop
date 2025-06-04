use clap::Parser;
use common::clap::AwsConfig;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct Cli {
    #[clap(long)]
    pub stage: common::clap::Stage,
    /// The address of the Kinesis stream endpoint
    #[clap(long, env = "KINESIS_ENDPOINT")]
    pub kinesis_endpoint: String,
    /// The name of the Kinesis stream containing journalist messages
    #[clap(long)]
    pub kinesis_u2j_stream: String,
    #[command(flatten)]
    pub aws_config: AwsConfig,
}
