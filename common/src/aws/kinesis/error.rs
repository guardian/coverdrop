use thiserror::Error;

/// Error variants for the Kinesis client
#[derive(Error, Debug)]
pub enum KinesisError {
    #[error("Could not find any shards in stream {0}")]
    NoShardsFound(String),
    #[error("Could not find shard id in stream {0}")]
    ShardIdError(String),
    #[error("Could not find shard iterator in shard {0}")]
    ShardIteratorError(String),
    #[error("Could not get records in shard iterator {0} in shard {1}")]
    GetRecordsError(String, String),
}
