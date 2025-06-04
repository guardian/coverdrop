use aws_config::default_provider::credentials::DefaultCredentialsChain;
use aws_sdk_kinesis::config::Region;
use aws_sdk_kinesis::primitives::Blob;
use aws_sdk_kinesis::Client;
use base64::prelude::*;
use common::api::models::messages::user_to_covernode_message::EncryptedUserToCoverNodeMessage;
use common::clap::AwsConfig;
use common::protocol::constants::USER_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN;

use crate::errors::KinesisPutRecordError;

#[derive(Clone)]
pub struct KinesisClient {
    inner: Client,
    user_to_journalist_stream: String,
}

impl KinesisClient {
    async fn build_credentials(profile: &Option<String>) -> DefaultCredentialsChain {
        let mut builder = DefaultCredentialsChain::builder();
        if let Some(profile) = profile {
            builder = builder.profile_name(profile);
        }

        builder.build().await
    }

    async fn build_inner(endpoint: &str, region: &str, profile: &Option<String>) -> Client {
        let region = Region::new(region.to_owned());
        let credentials_provider = KinesisClient::build_credentials(profile).await;

        let config = aws_sdk_kinesis::Config::builder()
            .behavior_version_latest()
            .endpoint_url(endpoint)
            .region(region)
            .credentials_provider(credentials_provider)
            .build();

        Client::from_conf(config)
    }

    pub async fn new(
        kinesis_endpoint: String,
        kinesis_u2j_endpoint: String,
        aws_config: &AwsConfig,
    ) -> KinesisClient {
        let inner =
            Self::build_inner(&kinesis_endpoint, &aws_config.region, &aws_config.profile).await;

        KinesisClient {
            inner,
            user_to_journalist_stream: kinesis_u2j_endpoint,
        }
    }

    /// Serializes and base64-encodes the u2j message before adding it to the Kinesis stream.
    pub async fn encode_and_put_u2j_message(
        &self,
        message: EncryptedUserToCoverNodeMessage,
    ) -> Result<(), KinesisPutRecordError> {
        assert_eq!(message.len(), USER_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN);
        let serialized = BASE64_STANDARD_NO_PAD.encode(message.as_bytes());

        let partition_key = serialized[..256].to_string();
        let data = Blob::new(serialized);

        self.inner
            .put_record()
            .stream_name(&self.user_to_journalist_stream)
            .partition_key(partition_key)
            .data(data)
            .send()
            .await?;

        Ok(())
    }
}
