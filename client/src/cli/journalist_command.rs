use clap::Subcommand;

#[derive(Subcommand)]
pub enum JournalistCommand {
    /// Create a new messaging key for a journalist
    GenerateMessagingKey,
    /// Send cover traffic as a journalist
    SendCover {
        /// The number of cover messages to send
        #[clap(default_value = "1")]
        number: usize,
    },
    /// Reply to a message as journalist
    ReplyToMessage {
        /// The public key associated with the message to reply to as retrieved by the `pull-dead-drops` command.
        /// You may use a prefix of the public key, but if multiple different matching keys are found this will result in an error.
        reply_to: String,
        /// The content of the message.
        message: String,
    },
    /// Download all dead drops to your vault
    PullDeadDrops,
    /// Starts an auto-reply service for the given journalist; option not available for users (sources)
    StartAutoReplyService,
    /// Print the contents of a vault, passwords for the vault can be provided with either the password or
    /// password-path flags. If neither is provided then you will be prompted for the password interactively.
    ReadVault,
}
