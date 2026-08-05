use std::collections::HashSet;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Duration;
use common::task::Task;
use tauri::AppHandle;

use crate::app_state::PublicInfo;
use crate::model::BackendToFrontendEvent;
use group_messaging_service::GroupMessagingService;

pub struct PollDeliveryService {
    group_messaging_service: Arc<GroupMessagingService>,
    public_info: PublicInfo,
    app_handle: AppHandle,
}

impl PollDeliveryService {
    pub fn new(
        // TODO make GroupMessagingService non-optional after all vaults have SentinelIdentities https://github.com/guardian/coverdrop-internal/issues/3885
        group_messaging_service: &Option<Arc<GroupMessagingService>>,
        public_info: &PublicInfo,
        app_handle: &AppHandle,
    ) -> Option<Self> {
        let Some(group_messaging_service) = group_messaging_service else {
            tracing::debug!("GroupMessagingService is unavailable, PollDeliveryService will not poll the delivery service.");
            return None;
        };
        Some(Self {
            group_messaging_service: group_messaging_service.clone(),
            public_info: public_info.clone(),
            app_handle: app_handle.clone(),
        })
    }
}

#[async_trait]
impl Task for PollDeliveryService {
    fn name(&self) -> &'static str {
        "poll_delivery_service"
    }

    async fn run(&self) -> anyhow::Result<()> {
        let public_info_guard = self.public_info.get().await;
        let Some(public_info) = public_info_guard.as_ref() else {
            tracing::debug!("No public info available, skipping pull from MLS delivery service");
            return Ok(());
        };

        let messages = &self
            .group_messaging_service
            .receive_and_store_messages(&public_info.keys)
            .await?;

        let num_messages_decrypted = &messages.len();

        let groups_ids_affected = messages
            .iter()
            .map(|message| message.group_id.clone())
            .collect::<HashSet<_>>();

        for group_id in groups_ids_affected {
            let _ = &self.app_handle.emit_group_change(group_id)?;
        }

        let log_message = format!("received {} messages", num_messages_decrypted);
        if num_messages_decrypted > &0 {
            tracing::info!(log_message);
        } else {
            tracing::debug!(log_message);
        }

        Ok(())
    }

    fn interval(&self) -> Duration {
        Duration::seconds(1)
    }
}
