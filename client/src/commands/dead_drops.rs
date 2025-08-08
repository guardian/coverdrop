use common::api::api_client::ApiClient;
use common::api::models::dead_drops::UnverifiedJournalistToUserDeadDropsList;

use common::client::mailbox::user_mailbox::UserMailbox;
use journalist_vault::{JournalistVault, VaultMessage};

pub async fn pull_user_dead_drops(
    api_client: &ApiClient,
) -> anyhow::Result<UnverifiedJournalistToUserDeadDropsList> {
    api_client.pull_user_dead_drops(0).await
}

pub fn print_user_dead_drops(mailbox: &UserMailbox) -> anyhow::Result<()> {
    println!("received_at\tfrom\tmessage");
    for message in mailbox.messages().iter() {
        println!(
            "{}\t{}\t{}",
            message.received_at,
            message.from,
            message.message.to_string()?
        );
    }
    Ok(())
}

pub async fn print_journalist_dead_drops(vault: &JournalistVault) -> anyhow::Result<()> {
    println!("received_at\tfrom\tmessage");
    for message in vault.messages().await? {
        match message {
            VaultMessage::U2J(msg) => {
                println!(
                    "{}\t{:?}\t{}",
                    msg.received_at, msg.user_pk.key, msg.message
                );
            }
            VaultMessage::J2U(msg) => {
                println!("{}\t{:?}\t{}", msg.sent_at, msg.user_pk.key, msg.message);
            }
        }
    }

    Ok(())
}
