use crate::commands::public_keys::get_public_keys;
use chrono::{DateTime, Utc};
use common::api::api_client::ApiClient;

use common::throttle::Throttle;
use journalist_vault::{JournalistVault, VaultMessage};

use common::aws::kinesis::client::KinesisClient;
use std::time::Duration;

use super::dead_drops::load_journalist_dead_drop_messages;
use super::messages::send_journalist_to_user_real_message;

const POLLING_RATE_MS: u64 = 1000;

pub async fn run_auto_reply_service<F>(
    api_client: ApiClient,
    kinesis_client: KinesisClient,
    vault: &JournalistVault,
    now_fn: F,
) -> anyhow::Result<()>
where
    F: Fn() -> DateTime<Utc>,
{
    let now = now_fn();
    let org_pks = vault.org_pks(now).await?;

    let keys_and_profiles = get_public_keys(&org_pks, &api_client, now).await?;

    let mut throttle = Throttle::new(Duration::from_millis(POLLING_RATE_MS));

    let mut last_printed_message_index = vault.messages().await?.len();

    loop {
        let now = now_fn();
        let max_dead_drop_id = vault.max_dead_drop_id().await?;
        let dead_drop_list = api_client
            .pull_all_journalist_dead_drops(max_dead_drop_id)
            .await?;

        load_journalist_dead_drop_messages(dead_drop_list, &keys_and_profiles.keys, vault, now)
            .await?;

        let messages = vault.messages().await?;

        if messages.len() > last_printed_message_index {
            let new_messages = messages[last_printed_message_index..].to_vec();

            for message in new_messages {
                match message {
                    VaultMessage::U2J(msg) => {
                        send_journalist_to_user_real_message(
                            &kinesis_client,
                            &keys_and_profiles.keys,
                            vault,
                            &msg.user_pk,
                            "Your message has been received and is being reviewed.",
                            now,
                        )
                        .await?;
                    }
                    VaultMessage::J2U(_) => {}
                }
            }

            last_printed_message_index = messages.len();
        }

        throttle.wait().await;
    }
}
