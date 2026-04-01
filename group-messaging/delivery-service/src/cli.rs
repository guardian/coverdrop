use clap::Parser;
use common::clap::{CliSecret, PostgresConnectionStringRedactor, Stage};
use reqwest::Url;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct Cli {
    #[clap(long)]
    pub stage: Stage,

    /// The URL of the API
    #[clap(long, env = "API_URL")]
    pub api_url: Url,

    /// The path to the Postgres database. If not set, the application will try to
    /// read it from the COVERDROP_DELIVERY_SERVICE_DB_URL environment variable
    #[clap(long, env = "COVERDROP_DELIVERY_SERVICE_DB_URL")]
    pub db_url: CliSecret<String, PostgresConnectionStringRedactor>,
}
