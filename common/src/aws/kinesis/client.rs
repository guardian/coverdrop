use std::collections::HashMap;

use super::error::KinesisError;
use super::models::checkpoint::{
    Checkpoints, CheckpointsJson, EncryptedJournalistToCoverNodeMessageWithCheckpointsJson,
    EncryptedUserToCoverNodeMessageWithCheckpointsJson, StoredCheckpoints,
};
use crate::api::models::messages::{
    journalist_to_covernode_message::EncryptedJournalistToCoverNodeMessage,
    user_to_covernode_message::EncryptedUserToCoverNodeMessage,
};
use crate::protocol::constants::JOURNALIST_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN;
use aws_config::default_provider::credentials::DefaultCredentialsChain;
use aws_config::timeout::TimeoutConfig;
use aws_sdk_kinesis::config::Region;
use aws_sdk_kinesis::primitives::Blob;
use aws_sdk_kinesis::types::{Record, ShardIteratorType};
use aws_sdk_kinesis::Client;
use base64::prelude::*;
use chrono::{DateTime, Duration, Utc};
use std::time::Duration as StdDuration;

use crate::clap::{AwsConfig, KinesisConfig};
use itertools::Itertools;
#[cfg(feature = "test-utils")]
use num_bigint::BigInt;

pub enum StreamKind {
    UserToJournalist,
    JournalistToUser,
}

// TODO this client is coupled to the CoverNode's use of Kinesis. Should this
// move to the CoverNode workspace or become a more general client?
#[derive(Clone)]
pub struct KinesisClient {
    inner: Client,

    user_to_journalist_stream: String,
    journalist_to_user_stream: String,

    // Shard iterators are tracked as a map of ShardId to a tuple of the ShardIterator
    // and a datetime when the iterator was created. This allows us to refresh shard
    // iterators that are too old.
    next_user_to_journalist_shard_iterators: HashMap<String, (String, DateTime<Utc>)>,
    next_journalist_to_user_shard_iterators: HashMap<String, (String, DateTime<Utc>)>,

    user_to_journalist_checkpoints: Checkpoints,
    journalist_to_user_checkpoints: Checkpoints,
}

impl KinesisClient {
    /// Slightly imperfect preflight checks on the kinesis stream, panics if you don't have valid credentials.
    /// There are failure modes this won't catch but will give a much more meaningful message in the common case
    /// which is when credentials are missing.
    async fn preflight_check(stream_name: &str, client: &Client) {
        tracing::debug!("Starting preflight check");

        client
            .list_shards()
            .stream_name(stream_name)
            .send()
            .await
            .expect("Read kinesis shards, do you have credentials loaded?");

        tracing::debug!("Preflight check successful");
    }

    async fn build_credentials(profile: &Option<String>) -> DefaultCredentialsChain {
        let mut builder = DefaultCredentialsChain::builder();
        if let Some(profile) = profile {
            builder = builder.profile_name(profile);
        }

        builder.build().await
    }

    async fn build_inner(
        endpoint: &str,
        region: &str,
        profile: &Option<String>,
        active_streams: Vec<String>,
    ) -> Client {
        let region = Region::new(region.to_owned());
        let credentials_provider = KinesisClient::build_credentials(profile).await;

        let timeout_config = TimeoutConfig::builder()
            .operation_timeout(StdDuration::from_secs(60))
            .build();

        let config = aws_sdk_kinesis::Config::builder()
            .behavior_version_latest()
            .endpoint_url(endpoint)
            .region(region)
            .credentials_provider(credentials_provider)
            .timeout_config(timeout_config)
            .build();

        let client = Client::from_conf(config);

        for active_stream in active_streams {
            Self::preflight_check(&active_stream, &client).await;
        }

        client
    }

    pub async fn new(
        kinesis_config: &KinesisConfig,
        aws_config: &AwsConfig,
        active_streams: Vec<String>,
    ) -> KinesisClient {
        let checkpoints = StoredCheckpoints {
            user_to_journalist_checkpoints: Checkpoints::new(),
            journalist_to_user_checkpoints: Checkpoints::new(),
        };

        Self::new_with_checkpoints(kinesis_config, aws_config, active_streams, checkpoints).await
    }

    pub async fn new_with_checkpoints(
        kinesis_config: &KinesisConfig,
        aws_config: &AwsConfig,
        active_streams: Vec<String>,
        stored_checkpoints: StoredCheckpoints,
    ) -> KinesisClient {
        let inner = Self::build_inner(
            &kinesis_config.endpoint,
            &aws_config.region,
            &aws_config.profile,
            active_streams,
        )
        .await;

        let user_to_journalist_stream = kinesis_config.user_stream.to_owned();
        let journalist_to_user_stream = kinesis_config.journalist_stream.to_owned();

        KinesisClient {
            inner,
            user_to_journalist_stream,
            journalist_to_user_stream,
            next_user_to_journalist_shard_iterators: HashMap::new(),
            next_journalist_to_user_shard_iterators: HashMap::new(),
            user_to_journalist_checkpoints: stored_checkpoints.user_to_journalist_checkpoints,
            journalist_to_user_checkpoints: stored_checkpoints.journalist_to_user_checkpoints,
        }
    }

    fn get_partition_key(bytes: &[u8]) -> String {
        // Kinesis partition keys have a maximum length of 256, so we need
        // to slice the encoded String to avoid overflows
        BASE64_STANDARD.encode(bytes)[..256].to_string()
    }

    /// Serializes and base64-encodes the journalist message before adding it to the Kinesis stream.
    pub async fn encode_and_put_journalist_message(
        &self,
        message: EncryptedJournalistToCoverNodeMessage,
    ) -> anyhow::Result<()> {
        assert_eq!(message.len(), JOURNALIST_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN);
        let serialized = BASE64_STANDARD_NO_PAD.encode(message.as_bytes());

        let partition_key = Self::get_partition_key(message.as_bytes());
        let data = Blob::new(serialized);

        self.inner
            .put_record()
            .stream_name(&self.journalist_to_user_stream)
            .partition_key(partition_key)
            .data(data)
            .send()
            .await?;

        Ok(())
    }

    /// Private helper function to process stream messages.
    async fn read_messages<F, T>(
        &mut self,
        stream_kind: StreamKind,
        limit: i32,
        func: F,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Vec<T>>
    where
        F: Fn(&Record, &Checkpoints) -> T,
    {
        let (stream_name, checkpoints, next_shard_iterators) = match stream_kind {
            StreamKind::UserToJournalist => (
                &self.user_to_journalist_stream,
                &mut self.user_to_journalist_checkpoints,
                &mut self.next_user_to_journalist_shard_iterators,
            ),
            StreamKind::JournalistToUser => (
                &self.journalist_to_user_stream,
                &mut self.journalist_to_user_checkpoints,
                &mut self.next_journalist_to_user_shard_iterators,
            ),
        };

        tracing::debug!("Fetching list of shards");

        let shards = self
            .inner
            .list_shards()
            .stream_name(stream_name)
            .send()
            .await?;

        let shard_ids = shards
            .shards
            .ok_or_else(|| KinesisError::NoShardsFound(stream_name.into()))?;

        tracing::debug!("Got {} shards for {}", shard_ids.len(), stream_name);

        // It's important to sort here so that we process the shards in the same order after a crash
        let shard_ids = shard_ids.iter().map(|shard| shard.shard_id()).sorted();

        let mut records: Vec<T> = vec![];

        // Shards expire after 5 minutes in AWS so we undershoot that a bit
        // here so we have some leeway before an error occurs
        const SHARD_ITERATOR_TTL: Duration = Duration::minutes(4);

        for shard_id in shard_ids {
            tracing::debug!("Getting shard iterator for {}", shard_id);

            let shard_iterator = match next_shard_iterators.get(shard_id) {
                // If we have a shard iterator already and it is less than $SHARD_EXPIRY_DURATION old
                Some((shard_iterator, created_at))
                    if now.signed_duration_since(created_at).abs() <= SHARD_ITERATOR_TTL =>
                {
                    tracing::debug!("Using existing shard iterator {}", shard_iterator);
                    shard_iterator
                }

                // Either we have no shard iterator or we do and it's *more* than $SHARD_EXPIRY_DURATION old
                _ => {
                    tracing::debug!("No existing valid shard iterator, creating a new one");

                    let new_shard_iterator = self
                        .inner
                        .get_shard_iterator()
                        .stream_name(stream_name)
                        .shard_id(shard_id);

                    let new_shard_iterator = match checkpoints.get(shard_id) {
                        // Start reading from checkpoint if sequence number is present
                        Some(starting_sequence_number) => {
                            tracing::info!(
                                "Creating shard iterator for {} from sequence number: {}",
                                shard_id,
                                starting_sequence_number,
                            );
                            new_shard_iterator
                                .shard_iterator_type(ShardIteratorType::AfterSequenceNumber)
                                .starting_sequence_number(starting_sequence_number.to_string())
                        }
                        // Else, read from the oldest data record available in the shard
                        None => {
                            tracing::info!(
                                "Creating shard iterator for {} using trim horizon",
                                shard_id,
                            );
                            new_shard_iterator.shard_iterator_type(ShardIteratorType::TrimHorizon)
                        }
                    };

                    let new_shard_iterator = new_shard_iterator.send().await?;
                    let new_shard_iterator = new_shard_iterator
                        .shard_iterator
                        .ok_or_else(|| KinesisError::ShardIteratorError(shard_id.to_owned()))?;

                    tracing::debug!("Got shard iterator {}", new_shard_iterator);

                    // To prevent creating new shard iterators every loop when a shard is not
                    // receiving any messages we immedietly insert it into our iterators map
                    // and then return it as a reference
                    next_shard_iterators.insert(shard_id.to_string(), (new_shard_iterator, now));

                    // SAFETY: This unwrap should be safe because we just inserted the iterator
                    // and this function owns `&mut self` so it cannot be modified elsewhere.
                    next_shard_iterators
                        .get(shard_id)
                        .map(|(i, _)| i)
                        .expect("Shard iterator should exist in map because we just inserted it")
                }
            };

            let get_records_output = self
                .inner
                .get_records()
                .shard_iterator(shard_iterator)
                .limit(limit)
                .send()
                .await?;

            tracing::debug!(
                "Got output from get-records, {} records",
                get_records_output.records().len()
            );

            let mut record_count = 0;

            let shard_records = get_records_output.records.iter().map(|record| {
                record_count += 1;

                tracing::trace!(
                    "Checkpointing shard_id: {}, sequence_number: {}",
                    shard_id,
                    record.sequence_number()
                );

                // Update the in-memory checkpoint for this shard
                //
                // This *must* be done for every record since every record holds the checkpoints
                // for every shard. Ideally a record would be paired with a (shard_id, sequence_number)
                // tuple and the checkpoint map would be flattened out at the point of publication.
                checkpoints.insert(shard_id.into(), record.sequence_number().into());

                func(record, checkpoints)
            });

            records.extend(shard_records);

            // If the get_records output had a next iterator then place that in the map of
            // next_shard_iterators
            if let Some(next_shard_iterator) = get_records_output.next_shard_iterator() {
                next_shard_iterators
                    .insert(shard_id.to_string(), (next_shard_iterator.to_string(), now));
            }
        }

        Ok(records)
    }

    pub async fn read_user_messages(
        &mut self,
        limit: i32,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Vec<anyhow::Result<EncryptedUserToCoverNodeMessageWithCheckpointsJson>>>
    {
        self.read_messages(
            StreamKind::UserToJournalist,
            limit,
            |record, checkpoints| {
                let data = record.data();

                let Ok(data) = BASE64_STANDARD_NO_PAD.decode(data) else {
                    anyhow::bail!("Error decoding user message");
                };

                let message = EncryptedUserToCoverNodeMessage::from_vec_unchecked(data);
                let checkpoints_json = CheckpointsJson::new(checkpoints)?;

                Ok(EncryptedUserToCoverNodeMessageWithCheckpointsJson {
                    message,
                    checkpoints_json,
                })
            },
            now,
        )
        .await
    }

    pub async fn read_journalist_messages(
        &mut self,
        limit: i32,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Vec<anyhow::Result<EncryptedJournalistToCoverNodeMessageWithCheckpointsJson>>>
    {
        self.read_messages(
            StreamKind::JournalistToUser,
            limit,
            |record, checkpoints| {
                let data = record.data();

                let Ok(data) = BASE64_STANDARD_NO_PAD.decode(data) else {
                    anyhow::bail!("Error decoding journalist message");
                };

                let message = EncryptedJournalistToCoverNodeMessage::from_vec_unchecked(data);
                let checkpoints_json = CheckpointsJson::new(checkpoints)?;

                Ok(EncryptedJournalistToCoverNodeMessageWithCheckpointsJson {
                    message,
                    checkpoints_json,
                })
            },
            now,
        )
        .await
    }

    // Features only used in the integration tests to simulate Kinesis splitting/merging shards.
    // This would normally done by an infrastructure service and would not be controlled directly by
    // any CoverDrop service.

    #[cfg(feature = "test-utils")]
    async fn split_shard(&self, stream_kind: StreamKind) -> anyhow::Result<()> {
        let stream_name = match stream_kind {
            StreamKind::UserToJournalist => &self.user_to_journalist_stream,
            StreamKind::JournalistToUser => &self.journalist_to_user_stream,
        };

        let shards = self
            .inner
            .list_shards()
            .stream_name(stream_name)
            .send()
            .await?;

        tracing::debug!("Got shards for {}", stream_name);

        let shard = shards
            .shards
            .ok_or_else(|| KinesisError::NoShardsFound(stream_name.into()))?;

        let shard = shard.first().ok_or(anyhow::anyhow!("No shard found"))?;

        let Some(hash_key_range) = shard.hash_key_range() else {
            anyhow::bail!("No hashkey range found on shard")
        };

        let Ok(starting_hash_key) = hash_key_range.starting_hash_key.parse::<BigInt>() else {
            anyhow::bail!("Could not parse starting hash key big int")
        };

        let Ok(ending_hash_key) = hash_key_range.ending_hash_key.parse::<BigInt>() else {
            anyhow::bail!("Could not parse ending hash key big int")
        };

        let new_starting_hash_key: BigInt = (starting_hash_key + ending_hash_key) / 2;

        self.inner
            .split_shard()
            .stream_name(stream_name)
            .shard_to_split(shard.shard_id())
            .new_starting_hash_key(new_starting_hash_key.to_string())
            .send()
            .await?;

        Ok(())
    }

    #[cfg(feature = "test-utils")]
    pub async fn split_journalist_to_user_shard(&self) -> anyhow::Result<()> {
        self.split_shard(StreamKind::JournalistToUser).await
    }

    #[cfg(feature = "test-utils")]
    pub async fn split_user_to_journalist_shard(&self) -> anyhow::Result<()> {
        self.split_shard(StreamKind::UserToJournalist).await
    }
}
