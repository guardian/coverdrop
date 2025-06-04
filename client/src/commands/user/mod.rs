pub mod dead_drops;
pub mod messages;

use std::path::PathBuf;

use common::api::api_client::ApiClient;
use common::time;
use common::u2j_appender::messaging_client::MessagingClient;
use hex::encode;

use crate::cli::UserCommand;
use crate::commands::{load_user_mailbox_from_args, print_mailbox_messages};

use self::{
    dead_drops::load_user_dead_drop_messages,
    messages::{send_user_to_journalist_cover_message, send_user_to_journalist_real_message},
};

#[allow(clippy::too_many_arguments)]
pub async fn handle_user_commands(
    mailbox_path: PathBuf,
    password: Option<String>,
    password_path: Option<PathBuf>,
    command: UserCommand,
    api_client: ApiClient,
) -> anyhow::Result<()> {
    let mut mailbox = load_user_mailbox_from_args(mailbox_path, password, password_path)?;

    let keys_and_profiles = api_client
        .get_public_keys()
        .await?
        .into_trusted(mailbox.org_pks(), time::now());

    match command {
        UserCommand::ReadMailbox => {
            println!(
                "Public Key: {}",
                encode(mailbox.secret.user_key_pair.raw_public_key().as_bytes())
            );
            println!(
                "Secret Key: {}",
                encode(mailbox.secret.user_key_pair.secret_key().to_bytes())
            );

            let messages = &mailbox.secret.messages;
            println!("{} Messages", messages.count());

            print_mailbox_messages(mailbox.secret.messages.iter())?;

            Ok(())
        }
        UserCommand::SendCover {
            messaging_url,
            number,
        } => {
            let messaging_client = MessagingClient::new(messaging_url);

            for _ in 0..number {
                send_user_to_journalist_cover_message(&messaging_client, &keys_and_profiles.keys)
                    .await?;
            }
            Ok(())
        }
        UserCommand::SendMessage {
            messaging_url,
            journalist_id,
            message,
        } => {
            let messaging_client = MessagingClient::new(messaging_url);

            send_user_to_journalist_real_message(
                &messaging_client,
                &mut mailbox,
                &keys_and_profiles.keys,
                &journalist_id,
                &message,
            )
            .await
        }
        UserCommand::PullDeadDrops => {
            let dead_drop_list = api_client
                .pull_user_dead_drops(mailbox.max_dead_drop_id())
                .await?;

            load_user_dead_drop_messages(
                &dead_drop_list,
                &keys_and_profiles.keys,
                &mut mailbox,
                time::now(),
            )?;

            Ok(())
        }
    }
}
