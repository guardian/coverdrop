use chrono::{DateTime, Utc};
use common::{
    client::mailbox::mailbox_message::{self},
    generators::NameGenerator,
};
use journalist_vault::VaultMessage;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize, TS)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[ts(export)]
pub enum UserStatus {
    Active,
    Muted,
}

impl UserStatus {
    pub fn from_mailbox_message_user_status(user_status: mailbox_message::UserStatus) -> Self {
        match user_status {
            mailbox_message::UserStatus::Muted => Self::Muted,
            mailbox_message::UserStatus::Active => Self::Active,
        }
    }
}

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
        from: String,
        from_display_name: String,
        message: String,
        received_at: DateTime<Utc>,
        read: bool,
        user_status: UserStatus,
        user_alias: Option<String>,
        user_description: Option<String>,
    },
    JournalistToUserMessage {
        id: i64,
        to: String,
        to_display_name: String,
        message: String,
        sent_at: DateTime<Utc>,
        is_sent: bool,
        user_status: UserStatus,
        user_alias: Option<String>,
        user_description: Option<String>,
    },
}

fn create_source_name(name_generator: &NameGenerator, reply_key: &[u8]) -> String {
    name_generator.name_from_bytes(reply_key.as_ref(), 2)
}

impl Message {
    pub fn from_vault_message(
        msg: &VaultMessage,
        name_generator: &NameGenerator,
    ) -> anyhow::Result<Self> {
        match msg {
            VaultMessage::U2J(msg) => {
                let message_text = msg.message.to_string()?;
                let user_status =
                    UserStatus::from_mailbox_message_user_status(msg.user_status.clone());
                Ok(Self::UserToJournalistMessage {
                    id: msg.id,
                    from: hex::encode(msg.user_pk.key),
                    from_display_name: create_source_name(
                        name_generator,
                        msg.user_pk.key.as_bytes(),
                    ),
                    message: message_text,
                    received_at: msg.received_at,
                    read: msg.read,
                    user_status,
                    user_alias: msg.user_alias.clone(),
                    user_description: msg.user_description.clone(),
                })
            }
            VaultMessage::J2U(msg) => {
                let message_text = msg.message.to_string()?;
                let user_status =
                    UserStatus::from_mailbox_message_user_status(msg.user_status.clone());
                Ok(Self::JournalistToUserMessage {
                    id: msg.id,
                    to: hex::encode(msg.user_pk.key),
                    to_display_name: create_source_name(name_generator, msg.user_pk.key.as_bytes()),
                    message: message_text,
                    sent_at: msg.sent_at,
                    is_sent: msg.is_sent,
                    user_status,
                    user_alias: msg.user_alias.clone(),
                    user_description: msg.user_description.clone(),
                })
            }
        }
    }
}
