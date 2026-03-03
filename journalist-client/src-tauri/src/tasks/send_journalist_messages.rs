use std::sync::Arc;

use async_trait::async_trait;
use chrono::Duration;
use common::{task::Task, time};
use coverdrop_service::JournalistCoverDropService;
use tauri::AppHandle;

use crate::app_state::PublicInfo;

use crate::model::BackendToFrontendEvent;

pub struct SendJournalistMessages {
    coverdrop_service: Arc<JournalistCoverDropService>,
    public_info: PublicInfo,
    app_handle: AppHandle,
}

impl SendJournalistMessages {
    pub fn new(
        coverdrop_service: &Arc<JournalistCoverDropService>,
        public_info: &PublicInfo,
        app_handle: &AppHandle,
    ) -> Self {
        Self {
            coverdrop_service: coverdrop_service.clone(),
            public_info: public_info.clone(),
            app_handle: app_handle.clone(),
        }
    }
}

#[async_trait]
impl Task for SendJournalistMessages {
    fn name(&self) -> &'static str {
        "send_journalist_message"
    }

    async fn run(&self) -> anyhow::Result<()> {
        let public_info = self.public_info.get().await;

        //
        // Always check if we've got public info before sending anything
        // even though it's not strictly needed to send a real message.
        //
        // This is to prevent any info leakage about a message being real in the
        // case where we've yet to pull public info but we sent a message.
        //

        if let Some(public_info) = public_info.as_ref() {
            let queue_length = self
                .coverdrop_service
                .dequeue_and_send_j2u_message(&public_info.keys, time::now())
                .await?;
            self.app_handle
                .emit_outbound_queue_length_event(queue_length)?;
        }

        Ok(())
    }

    fn interval(&self) -> Duration {
        Duration::minutes(1)
    }
}
