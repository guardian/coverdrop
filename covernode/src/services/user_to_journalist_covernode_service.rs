use crate::checkpoint::UserToJournalistDeadDropContentWithCheckpoints;
use crate::services::dead_drop_publishing::ToJournalistPublishingService;
use crate::services::decrypt_and_threshold::UserToJournalistDecryptionAndMixingService;
use crate::services::poll_messages::FromUserPollingService;
use crate::services::{CoverNodeServiceConfig, MPSC_CHANNEL_BOUND};
use common::aws::kinesis::models::checkpoint::EncryptedUserToCoverNodeMessageWithCheckpointsJson;
use common::tracing::log_task_result_exit;
use tokio::sync::mpsc;

pub struct UserToJournalistCoverNodeService {
    config: CoverNodeServiceConfig,
}

impl UserToJournalistCoverNodeService {
    pub fn new(config: CoverNodeServiceConfig) -> UserToJournalistCoverNodeService {
        UserToJournalistCoverNodeService { config }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        // create channels
        let (channel_polling_to_inner_sender, channel_polling_to_inner_receiver) =
            mpsc::channel::<EncryptedUserToCoverNodeMessageWithCheckpointsJson>(MPSC_CHANNEL_BOUND);
        let (channel_inner_to_publish_sender, channel_inner_to_publish_receiver) =
            mpsc::channel::<UserToJournalistDeadDropContentWithCheckpoints>(MPSC_CHANNEL_BOUND);

        // create polling service
        let mut polling_service = FromUserPollingService::new(
            self.config.kinesis_client.clone(),
            self.config.disable_stream_throttle,
        );
        let mut polling_service =
            tokio::spawn(async move { polling_service.run(channel_polling_to_inner_sender).await });

        // create decryption and threshold service
        let inner_service = UserToJournalistDecryptionAndMixingService::new(
            self.config.key_state.clone(),
            self.config.mixing_config,
        );

        let mut inner_service = tokio::spawn(async move {
            inner_service
                .run(
                    channel_polling_to_inner_receiver,
                    channel_inner_to_publish_sender,
                )
                .await
        });

        // create publishing service
        let publishing_service = ToJournalistPublishingService::new(
            self.config.key_state.clone(),
            self.config.api_client.clone(),
            self.config.checkpoint_path.clone(),
        );

        let mut publishing_service = tokio::spawn(async move {
            publishing_service
                .run(channel_inner_to_publish_receiver)
                .await
        });

        // block until the first service fails/exits; in that case we abort the others
        tokio::select! {
            r = (&mut polling_service) => {
                log_task_result_exit("U2J polling", r);

                inner_service.abort();
                publishing_service.abort();
            },
            r = (&mut inner_service) => {
                log_task_result_exit("U2J decrypt and threshold", r);

                polling_service.abort();
                publishing_service.abort();
            },
            r = (&mut publishing_service) => {
                log_task_result_exit("U2J dead drop publishing", r);

                polling_service.abort();
                inner_service.abort();
            },
        };

        Ok(())
    }
}
