use aws_config::default_provider::credentials::DefaultCredentialsChain;
use aws_sdk_kinesis::config::Region;
use aws_sdk_kinesis::primitives::Blob;
use aws_sdk_kinesis::Client;
use base64::prelude::*;
use common::api::models::messages::user_to_covernode_message::EncryptedUserToCoverNodeMessage;
use common::clap::AwsConfig;
use common::protocol::constants::USER_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN;

use crate::errors::AppError;

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
    #[allow(clippy::result_large_err)]
    pub async fn encode_and_put_u2j_message(
        &self,
        message: EncryptedUserToCoverNodeMessage,
    ) -> Result<(), AppError> {
        if message.len() != USER_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN {
            return Err(AppError::BadMessageSize);
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controllers::post_u2j_message;
    use aws_sdk_kinesis::config::{retry::RetryConfig, Credentials};
    use axum::{
        body::{to_bytes, Bytes},
        extract::State,
        http::StatusCode,
        response::IntoResponse,
        routing::post,
        Json, Router,
    };
    use serde_json::{json, Value};
    use tokio::{net::TcpListener, sync::mpsc};

    #[tokio::test]
    async fn rejects_invalid_sizes_without_appending_and_still_accepts_valid_messages() {
        let (sender, mut requests) = mpsc::channel::<Value>(1);
        let app = Router::new()
            .route(
                "/",
                post(
                    |State(sender): State<mpsc::Sender<Value>>, body: Bytes| async move {
                        let request = serde_json::from_slice(&body).unwrap();
                        sender.send(request).await.unwrap();
                        Json(json!({"ShardId": "shard-0", "SequenceNumber": "1"}))
                    },
                ),
            )
            .with_state(sender);
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = format!("http://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let config = aws_sdk_kinesis::Config::builder()
            .behavior_version_latest()
            .endpoint_url(endpoint)
            .region(Region::new("eu-west-1"))
            .credentials_provider(Credentials::new("test", "test", None, None, "test"))
            .retry_config(RetryConfig::disabled())
            .build();
        let client = KinesisClient {
            inner: Client::from_conf(config),
            user_to_journalist_stream: "test-stream".to_string(),
        };

        for size in [
            0,
            USER_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN - 1,
            USER_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN + 1,
        ] {
            let message = EncryptedUserToCoverNodeMessage::from_vec_unchecked(vec![0; size]);
            let body = Bytes::from(serde_json::to_vec(&message).unwrap());
            let error = post_u2j_message(None, State(client.clone()), body)
                .await
                .unwrap_err();
            assert!(matches!(error, AppError::BadMessageSize));
            let response = error.into_response();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            let body = to_bytes(response.into_body(), 1024).await.unwrap();
            assert_eq!(
                serde_json::from_slice::<Value>(&body).unwrap(),
                json!({"error": "Bad message size"})
            );
            assert!(matches!(
                requests.try_recv(),
                Err(mpsc::error::TryRecvError::Empty)
            ));
        }

        let message = EncryptedUserToCoverNodeMessage::from_vec_unchecked(vec![
            0;
            USER_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN
        ]);
        let serialized = BASE64_STANDARD_NO_PAD.encode(message.as_bytes());
        let body = Bytes::from(serde_json::to_vec(&message).unwrap());
        post_u2j_message(None, State(client), body).await.unwrap();
        let request = requests.recv().await.unwrap();
        assert_eq!(
            request["Data"],
            BASE64_STANDARD.encode(serialized.as_bytes())
        );
        assert_eq!(request["PartitionKey"], &serialized[..256]);

        server.abort();
        assert!(server.await.unwrap_err().is_cancelled());
    }
}
