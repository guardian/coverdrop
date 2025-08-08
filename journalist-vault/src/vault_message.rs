use chrono::{DateTime, Duration, Utc};
use common::{
    protocol::{constants::MESSAGE_VALID_FOR_DURATION_IN_SECONDS, keys::UserPublicKey},
    FixedSizeMessageText,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Serialize, Deserialize, TS)]
#[ts(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct U2JMessage {
    pub id: i64,
    pub user_pk: UserPublicKey,
    pub message: String,
    pub received_at: DateTime<Utc>,
    pub normal_expiry: DateTime<Utc>,
    pub custom_expiry: Option<DateTime<Utc>>,
    pub read: bool,
}

#[derive(Clone, Serialize, Deserialize, TS)]
#[ts(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct J2UMessage {
    pub id: i64,
    pub user_pk: UserPublicKey,
    pub message: String,
    pub is_sent: bool,
    pub sent_at: DateTime<Utc>,
    pub normal_expiry: DateTime<Utc>,
    pub custom_expiry: Option<DateTime<Utc>>,
}

impl U2JMessage {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: i64,
        from: UserPublicKey,
        message: FixedSizeMessageText,
        received_at: DateTime<Utc>,
        custom_expiry: Option<DateTime<Utc>>,
        read: bool,
    ) -> anyhow::Result<Self> {
        let message_retention_duration = Duration::seconds(MESSAGE_VALID_FOR_DURATION_IN_SECONDS);
        Ok(Self {
            id,
            user_pk: from,
            message: message.to_string()?,
            received_at,
            normal_expiry: received_at + message_retention_duration,
            custom_expiry,
            read,
        })
    }
}

impl J2UMessage {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: i64,
        to: UserPublicKey,
        message: FixedSizeMessageText,
        is_sent: bool,
        sent_at: DateTime<Utc>,
        custom_expiry: Option<DateTime<Utc>>,
    ) -> anyhow::Result<Self> {
        let message_retention_duration = Duration::seconds(MESSAGE_VALID_FOR_DURATION_IN_SECONDS);
        Ok(Self {
            id,
            user_pk: to,
            message: message.to_string()?,
            is_sent,
            sent_at,
            normal_expiry: sent_at + message_retention_duration,
            custom_expiry,
        })
    }
}

#[derive(Clone, Serialize, Deserialize, TS)]
#[serde(tag = "type")]
#[ts(export)]
#[ts(rename = "Message")]
pub enum VaultMessage {
    #[serde(rename = "userToJournalistMessage")]
    U2J(U2JMessage),
    #[serde(rename = "journalistToUserMessage")]
    J2U(J2UMessage),
}
