use crate::tls_serialized::TlsSerialized;
use chrono::{DateTime, Utc};
use common::api::models::sentinel_id::SentinelIdentity;
use openmls::prelude::KeyPackage;
use serde::{Deserialize, Serialize};

/// A key package paired with the client ID it belongs to
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPackageWithClientId {
    pub key_package: KeyPackage,
    pub client_id: SentinelIdentity,
}

/// TLS-serialized message content, with its auto-incrementing ID and the timestamp at which it was published.
/// Used by the delivery service to represent messages and to serialize/deserialize them when they are sent to clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMessage {
    pub message_id: i32,
    pub published_at: DateTime<Utc>,
    pub content: TlsSerialized,
}

/// Response from the send message endpoint, containing the server-assigned published_at timestamp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageResponse {
    pub published_at: DateTime<Utc>,
}
