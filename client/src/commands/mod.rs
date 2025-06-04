pub mod dead_drops;
pub mod journalist;
pub mod public_keys;
pub mod user;

use std::path::{Path, PathBuf};

use common::{
    clap::validate_password_from_args,
    client::mailbox::{mailbox_message::MailboxMessage, user_mailbox::UserMailbox},
};
use journalist_vault::JournalistVault;

pub fn load_user_mailbox_from_args(
    mailbox_path: impl AsRef<Path>,
    password: Option<String>,
    password_path: Option<PathBuf>,
) -> anyhow::Result<UserMailbox> {
    let valid_password = validate_password_from_args(password, password_path)?;

    UserMailbox::load(mailbox_path, &valid_password)
}

pub async fn load_journalist_vault_from_args(
    vault_path: impl AsRef<Path>,
    password: Option<String>,
    password_path: Option<PathBuf>,
) -> anyhow::Result<JournalistVault> {
    // Parse password
    let valid_password = validate_password_from_args(password, password_path)?;

    // Open mailbox
    JournalistVault::open(&vault_path, &valid_password).await
}

pub fn print_mailbox_messages<'a>(
    messages: impl Iterator<Item = &'a MailboxMessage>,
) -> anyhow::Result<()> {
    for mailbox_message in messages {
        let text = mailbox_message.message.to_string()?;

        println!("From: {}", mailbox_message.from);

        println!("Date: {}", mailbox_message.received_at);
        println!("Message: {text}");
        println!();
    }

    Ok(())
}
