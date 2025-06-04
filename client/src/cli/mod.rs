use clap::{Parser, Subcommand};
use common::clap::{AwsConfig, KinesisConfig};
use reqwest::Url;
use std::path::PathBuf;

mod journalist_command;
mod user_command;

pub use journalist_command::JournalistCommand;
pub use user_command::UserCommand;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct Cli {
    /// The address of the CoverDrop API server
    #[clap(long)]
    pub api_url: Url,
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Check to see how many characters you've got left for a given mesage
    CheckMessageLength {
        /// The message to check
        #[clap(long)]
        message: String,
    },
    /// Create a new mailbox, this will autogenerate and print a new password
    GenerateUser {
        /// The path to where you wish to create the new mailbox
        #[clap(long)]
        mailbox_path: PathBuf,
        /// Optionally, force a specific password
        #[clap(long)]
        password: Option<String>,
    },
    /// Subcommands as a user
    User {
        /// The path to the mailbox
        #[clap(long)]
        mailbox_path: PathBuf,

        /// Optionally, the password to unlock your mailbox.
        #[clap(long, global = true)]
        password: Option<String>,
        /// Optionally, path to a file which contains your mailbox's password.
        #[clap(long, conflicts_with = "password", global = true)]
        password_path: Option<PathBuf>,

        /// The user subcommand
        #[clap(subcommand)]
        command: UserCommand,
    },
    /// Subcommands as a journalist
    Journalist {
        /// The path to the journalist's mailbox
        #[clap(long)]
        vault_path: PathBuf,
        /// Optionally, the password to unlock your mailbox.
        #[clap(long, global = true)]
        password: Option<String>,
        /// Optionally, path to a file which contains your mailbox's password.
        #[clap(long, conflicts_with = "password", global = true)]
        password_path: Option<PathBuf>,

        #[command(flatten)]
        aws_config: AwsConfig,

        #[command(flatten)]
        kinesis_config: KinesisConfig,

        /// Journalist subcommand
        #[clap(subcommand)]
        command: JournalistCommand,
    },
}
