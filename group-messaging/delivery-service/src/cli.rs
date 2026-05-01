use clap::Parser;
use common::clap::{CliSecret, PostgresConnectionStringRedactor, Stage};
use reqwest::Url;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct Cli {
    #[clap(long, env = "STAGE")]
    pub stage: Stage,

    /// The URL of the API
    #[clap(long, env = "API_URL")]
    pub api_url: Url,

    /// The path to the Postgres database. If not set, the application will try to
    /// read it from the COVERDROP_DELIVERY_SERVICE_DB_URL environment variable.
    /// Either this or db_secret_arn must be provided.
    #[clap(long, env = "COVERDROP_DELIVERY_SERVICE_DB_URL")]
    pub db_url: Option<CliSecret<String, PostgresConnectionStringRedactor>>,

    /// The ARN of an AWS Secrets Manager secret containing the database credentials.
    /// If provided (and db_url is not set), the database URL will be constructed
    /// from the secret's username, password, host and port fields.
    #[clap(long, env = "COVERDROP_DELIVERY_SERVICE_DB_SECRET_ARN")]
    pub db_secret_arn: Option<String>,
}
