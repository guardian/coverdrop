use clap::Subcommand;
use common::api::models::journalist_id::JournalistIdentity;
use reqwest::Url;

#[derive(Subcommand)]
pub enum UserCommand {
    /// Print the contents of a mailbox, passwords for the mailbox can be provided with either the password or
    /// password-path flags. If neither is provided then you will be prompted for the password interactively.
    ReadMailbox,
    /// Send cover traffic as a user
    SendCover {
        /// The address of the U2J service forwarding messages to Kinesis
        #[clap(long)]
        messaging_url: Url,

        /// The number of cover messages to send
        #[clap(long, default_value = "1")]
        number: usize,
    },
    /// Send a message to the CoverDrop API as a user
    SendMessage {
        /// The address of the U2J service forwarding messages to Kinesis
        #[clap(long)]
        messaging_url: Url,

        /// The ID of the journalist's public key.
        #[clap(long)]
        journalist_id: JournalistIdentity,
        /// The content of the message.
        #[clap(long)]
        message: String,
    },
    /// Download all dead drops to your mailbox
    PullDeadDrops,
}
