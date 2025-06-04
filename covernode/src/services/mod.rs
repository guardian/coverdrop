use crate::key_state::KeyState;
use crate::mixing::mixing_strategy::MixingStrategyConfiguration;
use common::api::api_client::ApiClient;
use common::aws::kinesis::client::KinesisClient;
use reqwest::Url;
use std::path::PathBuf;

pub mod dead_drop_publishing;
pub mod decrypt_and_threshold;
pub mod journalist_to_user_covernode_service;
pub mod poll_messages;
pub mod server;
pub mod tasks;
pub mod user_to_journalist_covernode_service;

#[derive(Clone)]
pub struct CoverNodeServiceConfig {
    pub api_url: Url,
    pub key_state: KeyState,
    pub api_client: ApiClient,
    pub checkpoint_path: PathBuf,
    pub kinesis_client: KinesisClient,
    pub mixing_config: MixingStrategyConfiguration,
    pub disable_stream_throttle: bool,
}

/// The capacity of the two channels. This should be small enough to avoid the process go OOM. At
/// the same time it must be large enough such that slower decryption rounds do not externally
/// visibly affect the [PollingService] and [PublishingService].
///
/// Assuming a memory footprint of 10 KiB per message, we set it to 100,000 resulting in an overall
/// maximal memory demand of 2 * 50_000 * 10 KiB = 1 GiB.
const MPSC_CHANNEL_BOUND: usize = 50_000;
