use crate::checkpoint::JournalistToUserDeadDropContentWithCheckpoints;
use crate::services::dead_drop_publishing::ToUserPublishingService;
use crate::services::decrypt_and_threshold::JournalistToUserDecryptionAndMixingService;
use crate::services::poll_messages::FromJournalistPollingService;
use crate::services::{CoverNodeServiceConfig, MPSC_CHANNEL_BOUND};
use common::aws::kinesis::models::checkpoint::EncryptedJournalistToCoverNodeMessageWithCheckpointsJson;
use common::tracing::log_task_result_exit;
use tokio::sync::mpsc;

pub struct JournalistToUserCoverNodeService {
    config: CoverNodeServiceConfig,
}

impl JournalistToUserCoverNodeService {
    pub fn new(config: CoverNodeServiceConfig) -> JournalistToUserCoverNodeService {
        JournalistToUserCoverNodeService { config }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        // create channels
        let (channel_polling_to_inner_sender, channel_polling_to_inner_receiver) =
            mpsc::channel::<EncryptedJournalistToCoverNodeMessageWithCheckpointsJson>(
                MPSC_CHANNEL_BOUND,
            );

        let (channel_inner_to_publish_sender, channel_inner_to_publish_receiver) =
            mpsc::channel::<JournalistToUserDeadDropContentWithCheckpoints>(MPSC_CHANNEL_BOUND);

        // create polling service
        let mut polling_service = FromJournalistPollingService::new(
            self.config.kinesis_client.clone(),
            self.config.disable_stream_throttle,
        );

        let mut polling_service =
            tokio::spawn(async move { polling_service.run(channel_polling_to_inner_sender).await });

        // create decryption and threshold service
        let inner_service = JournalistToUserDecryptionAndMixingService::new(
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
        let publishing_service = ToUserPublishingService::new(
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
                log_task_result_exit("J2U polling", r);

                inner_service.abort();
                publishing_service.abort();
            },
            r = (&mut inner_service) => {
                log_task_result_exit("J2U decrypt and threshold", r);

                polling_service.abort();
                publishing_service.abort();
            },
            r = (&mut publishing_service) => {
                log_task_result_exit("J2U dead drop publishing", r);

                polling_service.abort();
                inner_service.abort();
            },
        };

        Ok(())
    }
}
