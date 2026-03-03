use std::collections::HashSet;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Duration;
use common::{
    client::mailbox::mailbox_message::UserStatus, protocol::keys::UserPublicKey, task::Task, time,
};
use tauri::AppHandle;

use crate::{app_state::PublicInfo, notifications::Notifications};
use coverdrop_service::JournalistCoverDropService;

use crate::model::BackendToFrontendEvent;

pub struct PullDeadDrops {
    app_handle: AppHandle,
    coverdrop_service: Arc<JournalistCoverDropService>,
    notifications: Notifications,
    public_info: PublicInfo,
}

impl PullDeadDrops {
    pub fn new(
        app_handle: &AppHandle,
        coverdrop_service: &Arc<JournalistCoverDropService>,
        notifications: &Notifications,
        public_info: &PublicInfo,
    ) -> Self {
        Self {
            app_handle: app_handle.clone(),
            coverdrop_service: coverdrop_service.clone(),
            notifications: notifications.clone(),
            public_info: public_info.clone(),
        }
    }
}

#[async_trait]
impl Task for PullDeadDrops {
    fn name(&self) -> &'static str {
        "pull_dead_drops"
    }

    async fn run(&self) -> anyhow::Result<()> {
        // Use the CoverDrop service to pull and decrypt messages
        let public_info_guard = self.public_info.get().await;
        let Some(public_info) = public_info_guard.as_ref() else {
            tracing::debug!("No public info available, skipping dead drop pull");
            return Ok(());
        };

        self.app_handle.emit_dead_drops_pull_started()?;

        let decrypted_messages = self
            .coverdrop_service
            .pull_and_decrypt_dead_drops(
                public_info,
                Some(move |remaining| {
                    let _ = self.app_handle.emit_dead_drops_remaining_event(remaining);
                }),
                time::now(),
            )
            .await?;
        tracing::info!(
            "pull_and_decrypt_dead_drops returned {} messages",
            decrypted_messages.len()
        );

        // Handle notifications for active users
        if !decrypted_messages.is_empty() {
            let users = self.coverdrop_service.get_users().await?;
            let active_users: HashSet<&UserPublicKey> = users
                .iter()
                .filter(|u| u.status == UserStatus::Active)
                .map(|u| &u.user_pk)
                .collect();

            let num_messages_from_active_users = decrypted_messages
                .iter()
                .filter(|m| active_users.contains(&&m.u2j_message.reply_key))
                .count();

            if num_messages_from_active_users > 0 {
                let notification_message = if num_messages_from_active_users == 1 {
                    "Received a message".to_owned()
                } else {
                    format!("Received {} messages", num_messages_from_active_users)
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
