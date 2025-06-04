use std::collections::HashSet;

use async_trait::async_trait;
use chrono::Duration;
use common::{
    api::{
        api_client::ApiClient,
        models::messages::user_to_journalist_message_with_dead_drop_id::UserToJournalistMessageWithDeadDropId,
    },
    client::mailbox::mailbox_message::UserStatus,
    protocol::{
        covernode::verify_user_to_journalist_dead_drop_list,
        journalist::get_decrypted_journalist_dead_drop_message, keys::UserPublicKey,
    },
    task::Task,
    time,
};
use journalist_vault::JournalistVault;

use crate::{app_state::PublicInfo, notifications::Notifications};

pub struct PullDeadDrops {
    api_client: ApiClient,
    vault: JournalistVault,
    public_info: PublicInfo,
    notifications: Notifications,
}

impl PullDeadDrops {
    pub fn new(
        api_client: &ApiClient,
        vault: &JournalistVault,
        public_info: &PublicInfo,
        notifications: &Notifications,
    ) -> Self {
        Self {
            api_client: api_client.clone(),
            vault: vault.clone(),
            public_info: public_info.clone(),
            notifications: notifications.clone(),
        }
    }
}

#[async_trait]
impl Task for PullDeadDrops {
    fn name(&self) -> &'static str {
        "pull_dead_drops"
    }

    async fn run(&self) -> anyhow::Result<()> {
        let public_info = self.public_info.get().await;

        if let Some(public_info) = public_info.as_ref() {
            let now = time::now();
            let keys = &public_info.keys;

            let ids_greater_than = self.vault.max_dead_drop_id().await?;

            tracing::info!(
                "Pulling dead drops with ID greater than {}",
                ids_greater_than
            );

            let dead_drop_list = self
                .api_client
                .pull_journalist_dead_drops(ids_greater_than, None)
                .await?;

            let Some(max_dead_drop_id) = dead_drop_list
                .dead_drops
                .iter()
                .max_by_key(|d| d.id)
                .map(|d| d.id)
            else {
                tracing::info!("No dead drops in dead drop list");

                return Ok(());
            };

            let dead_drops =
                verify_user_to_journalist_dead_drop_list(keys, dead_drop_list, time::now());

            tracing::info!("Found {} dead drops", dead_drops.len());

            let journalist_msg_key_pairs = self
                .vault
                .msg_key_pairs_for_decryption(time::now())
                .await?
                .collect::<Vec<_>>();

            let covernode_msg_pks = keys
                .covernode_msg_pk_iter()
                .map(|(_, msg_pk)| msg_pk)
                .collect::<Vec<_>>();

            let decrypted_messages: Vec<UserToJournalistMessageWithDeadDropId> = dead_drops
                .iter()
                .flat_map(|dead_drop| {
                    dead_drop
                        .data
                        .messages
                        .iter()
                        .filter_map(|encrypted_message| {
                            get_decrypted_journalist_dead_drop_message(
                                &covernode_msg_pks,
                                &journalist_msg_key_pairs,
                                encrypted_message,
                                dead_drop.id,
                            )
                        })
                })
                .collect();

            let users = self.vault.users().await?;
            let active_users: HashSet<&UserPublicKey> = users
                .iter()
                .filter(|u| u.status == UserStatus::Active)
                .map(|u| &u.user_pk)
                .collect();

            self.vault
                .add_messages_from_user_to_journalist_and_update_max_dead_drop_id(
                    &decrypted_messages,
                    max_dead_drop_id,
                    now,
                )
                .await?;

            let messages_from_active_users: Vec<&UserToJournalistMessageWithDeadDropId> =
                decrypted_messages
                    .iter()
                    .filter(|m| active_users.contains(&&m.u2j_message.reply_key))
                    .collect();

            if !messages_from_active_users.is_empty() {
                let notification_message = if messages_from_active_users.len() == 1 {
                    "Received a message".to_owned()
                } else {
                    format!("Received {} messages", messages_from_active_users.len())
                };

                self.notifications
                    .send_with_default_title(notification_message)
                    .await;
            }
        }

        Ok(())
    }

    fn interval(&self) -> Duration {
        Duration::seconds(15)
    }
}
