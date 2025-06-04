use clap::Parser;
use common::{
    clap::{CliSecret, PlainRedactor},
    task::RunnerMode,
};
use reqwest::Url;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct Cli {
    // The stage the Identity API is running in
    #[clap(long, value_enum)]
    pub stage: common::clap::Stage,
    /// The address of the CoverDrop API server
    #[clap(long)]
    pub api_url: Url,
    /// The path to the directory where the various keys are.
    #[clap(long)]
    pub keys_path: PathBuf,
    /// The path to the SQLCipher database used for storing sensitive key material
    #[clap(long)]
    pub db_path: PathBuf,
    /// The password for the SQLCipher database
    #[clap(long)]
    pub db_password: CliSecret<String, PlainRedactor>,

    /// The mode to start the task runner for either time based execution or manually triggered
    /// via a web server.
    #[clap(long, default_value = "timer")]
    pub task_runner_mode: RunnerMode,
}
