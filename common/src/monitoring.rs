use crate::api::models::realms::Realm;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum MonitoringMessage {
    ReceivedMessage {
        from: Realm,
    },
    PublishedDeadDrop {
        to: Realm,
    },
    MixThresholdLevel {
        from: Realm,
        to: Realm,
        current_level: usize,
        mixing_input_threshold: usize,
    },
    TracingEvent {
        level: String,
        source: String,
        message: Option<String>,
    },
}

impl MonitoringMessage {
    pub fn serialize(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap()
    }
}
