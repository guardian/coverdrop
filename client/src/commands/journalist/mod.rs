mod auto_reply;
pub mod dead_drops;
pub mod keys;
pub mod messages;

use std::path::PathBuf;

use common::aws::kinesis::client::KinesisClient;
use common::{
    api::api_client::ApiClient, crypto::keys::signing::traits::PublicSigningKey,
    protocol::keys::UserPublicKey, time,
};

use crate::commands::dead_drops::print_journalist_dead_drops;
use crate::{
    cli::JournalistCommand, commands::journalist::dead_drops::load_journalist_dead_drop_messages,
    error::Error,
};

use self::{
    auto_reply::run_auto_reply_service,
    messages::{send_journalist_to_user_cover_message, send_journalist_to_user_real_message},
};

use super::load_journalist_vault_from_args;

pub async fn handle_journalist_command(
    vault_path: PathBuf,
    password: Option<String>,
    password_path: Option<PathBuf>,
    command: JournalistCommand,
    api_client: ApiClient,
    kinesis_client: KinesisClient,
) -> anyhow::Result<()> {
    let vault = load_journalist_vault_from_args(&vault_path, password, password_path).await?;

    let org_pks = vault.org_pks(time::now()).await?;

    let keys_and_profiles = api_client
        .get_public_keys()
        .await?
        .into_trusted(&org_pks, time::now());

    match command {
        JournalistCommand::SendCover { number } => {
            for _ in 0..number {
                send_journalist_to_user_cover_message(&kinesis_client, &keys_and_profiles.keys)
                    .await?;
            }
            Ok(())
        }
        JournalistCommand::ReplyToMessage { reply_to, message } => {
            let reply_to_key_prefix = reply_to.to_lowercase();

            let user_keys = vault.user_keys().await?;

            // Given a key prefix we should find the possible users
            let user_pk = {
                let mut candidate_user_keys: Vec<UserPublicKey> = user_keys
                    .into_iter()
                    .map(|key| {
                        let key_hex = hex::encode(key.key.as_bytes());
                        (key_hex, key)
                    })
                    .filter_map(|(key_hex, key)| {
                        if key_hex.starts_with(&reply_to_key_prefix) {
                            Some(key)
                        } else {
                            None
                        }
                    })
                    .collect();

                candidate_user_keys.dedup();

                if candidate_user_keys.len() > 1 {
                    Err(Error::MultiplePublicKeys)
                } else if let Some(key) = candidate_user_keys.pop() {
                    Ok(key)
                } else {
                    Err(Error::PublicKeyNotFound)
                }
            }?;

            send_journalist_to_user_real_message(
                &kinesis_client,
                &keys_and_profiles.keys,
                &vault,
                &user_pk,
                &message,
                time::now(),
            )
            .await
        }
        JournalistCommand::PullDeadDrops => {
            let max_dead_drop_id = vault.max_dead_drop_id().await?;

            let dead_drop_list = api_client
                .pull_all_journalist_dead_drops(max_dead_drop_id)
                .await?;

            load_journalist_dead_drop_messages(
                dead_drop_list,
                &keys_and_profiles.keys,
                &vault,
                time::now(),
            )
            .await?;

            Ok(())
        }
        JournalistCommand::StartAutoReplyService => {
            // The auto-reply service runs in a loop so has to get the now time more than once
            run_auto_reply_service(api_client, kinesis_client, &vault, time::now).await
        }
        JournalistCommand::GenerateMessagingKey => {
            vault
                .generate_msg_key_pair_and_upload_pk(&api_client, time::now())
                .await
        }
        JournalistCommand::ReadVault => {
            let id_key_pairs = vault.id_key_pairs(time::now()).await?;

            println!("Journalist ID key pair");
            for (idx, id_key_pair) in id_key_pairs.enumerate() {
                println!("    Key Pair #{idx}");
                println!(
                    "    Public Key: {}",
                    hex::encode(id_key_pair.raw_public_key().as_bytes())
                );
                println!(
                    "    Secret Key: {}",
                    hex::encode(id_key_pair.secret_key.to_bytes())
                );
            }

            let msg_key_pairs = vault.msg_key_pairs_for_decryption(time::now()).await?;

            println!("Journalist messaging key pair");
            for (idx, msg_key_pair) in msg_key_pairs.enumerate() {
                println!("    Key Pair #{idx}");
                println!(
                    "    Public Key: {}",
                    hex::encode(msg_key_pair.raw_public_key().as_bytes())
                );
                println!(
                    "    Secret Key: {}",
                    hex::encode(msg_key_pair.secret_key().to_bytes())
                );
            }

            print_journalist_dead_drops(&vault).await?;

            Ok(())
        }
    }
}
