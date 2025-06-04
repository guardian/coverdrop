use async_trait::async_trait;
use chrono::Duration;
use common::{
    api::api_client::ApiClient,
    protocol::{
        constants::MINUTE_IN_SECONDS,
        journalist::new_encrypted_cover_message_from_journalist_via_covernode,
    },
    task::Task,
    time,
};
use journalist_vault::JournalistVault;

use crate::app_state::PublicInfo;

pub struct SendJournalistMessages {
    api_client: ApiClient,
    vault: JournalistVault,
    public_info: PublicInfo,
}

impl SendJournalistMessages {
    pub fn new(api_client: &ApiClient, vault: &JournalistVault, public_info: &PublicInfo) -> Self {
        Self {
            api_client: api_client.clone(),
            vault: vault.clone(),
            public_info: public_info.clone(),
        }
    }
}

#[async_trait]
impl Task for SendJournalistMessages {
    fn name(&self) -> &'static str {
        "send_journalist_message"
    }

    async fn run(&self) -> anyhow::Result<()> {
        let now = time::now();

        let public_info = self.public_info.get().await;

        //
        // Always check if we've got public info before sending anything
        // even though it's not strictly needed to send a real message.
        //
        // This is to prevent any info leakage about a message being real in the
        // case where we've yet to pull public info but we sent a message.
        //

        if let Some(public_info) = public_info.as_ref() {
            let Some(id_key_pair) = self.vault.latest_id_key_pair(now).await? else {
                anyhow::bail!("No ID key pair found in vault");
            };

            if let Ok(Some(message)) = self.vault.head_queue_message().await {
                tracing::debug!("Found message in vault queue");

                self.api_client
                    .post_journalist_msg(message.message, &id_key_pair, now)
                    .await?;

                tracing::debug!("Posting message was successful, deleting message from queue");

                self.vault.delete_queue_message(message.id).await?;

                tracing::debug!("Successfully deleted message from queue");
            } else {
                let keys = &public_info.keys;

                tracing::debug!("No message in vault queue, creating and sending cover message");

                let message = new_encrypted_cover_message_from_journalist_via_covernode(keys)?;

                self.api_client
                    .post_journalist_msg(message, &id_key_pair, now)
                    .await?;

                tracing::debug!("Posting message was successful");
            }
        }

        Ok(())
    }

    fn interval(&self) -> Duration {
        Duration::seconds(MINUTE_IN_SECONDS)
    }
}
