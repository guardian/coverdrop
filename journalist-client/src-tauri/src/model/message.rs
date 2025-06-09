use chrono::{DateTime, Utc};
use journalist_vault::VaultMessage;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize, TS)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "type"
)]
#[ts(export)]
pub enum Message {
    UserToJournalistMessage {
        id: i64,
        user_pk: String,
        message: String,
        received_at: DateTime<Utc>,
        read: bool,
    },
    JournalistToUserMessage {
        id: i64,
        user_pk: String,
        message: String,
        sent_at: DateTime<Utc>,
        is_sent: bool,
    },
}

impl Message {
    pub fn from_vault_message(msg: &VaultMessage) -> anyhow::Result<Self> {
        match msg {
            VaultMessage::U2J(msg) => {
                let message_text = msg.message.to_string()?;
                Ok(Self::UserToJournalistMessage {
                    id: msg.id,
                    user_pk: hex::encode(msg.user_pk.key),
                    message: message_text,
                    received_at: msg.received_at,
                    read: msg.read,
                })
            }
            VaultMessage::J2U(msg) => {
                let message_text = msg.message.to_string()?;
                Ok(Self::JournalistToUserMessage {
                    id: msg.id,
                    user_pk: hex::encode(msg.user_pk.key),
                    message: message_text,
                    sent_at: msg.sent_at,
                    is_sent: msg.is_sent,
                })
            }
        }
    }
}
