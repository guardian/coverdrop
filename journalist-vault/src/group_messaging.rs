use std::fmt::{Display, Formatter};

use chrono::{DateTime, Utc};
use common::api::models::sentinel_id::SentinelIdentity;
use serde::{Deserialize, Serialize};
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::sqlite::SqliteTypeInfo;
use ts_rs::TS;

#[derive(TS)]
#[ts(export)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash, sqlx::Type)]
#[sqlx(transparent)]
pub struct GroupId(String);

pub use common::api::models::message_id::MessageId;

impl GroupId {
    pub fn new(id: String) -> Self {
        Self(id)
    }
}

impl Display for GroupId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents the content of a message in a group chat to be stored in the vault and displayed in the UI.
/// It includes variants that are never directly sent via the DS (UserAdded, UserRemoved). These two variants
/// are the user-facing representations of MLS commit messages.
/// Stored as JSON in the database via manual sqlx trait implementations.
#[derive(TS)]
#[ts(export)]
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum GroupMessageContent {
    Text(String),
    IsTyping(bool),
    UsersAdded(Vec<SentinelIdentity>),
    UsersRemoved(Vec<SentinelIdentity>),
    GroupNameChanged(String), // the new group name
    GroupDescriptionChanged(String),
    GroupInfo {
        display_name: String,
        description: String,
    },
}

impl sqlx::Type<sqlx::Sqlite> for GroupMessageContent {
    fn type_info() -> SqliteTypeInfo {
        <String as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Sqlite> for GroupMessageContent {
    fn encode_by_ref(
        &self,
        buf: &mut <sqlx::Sqlite as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<IsNull, BoxDynError> {
        let json = serde_json::to_string(self)?;
        <String as sqlx::Encode<'q, sqlx::Sqlite>>::encode(json, buf)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Sqlite> for GroupMessageContent {
    fn decode(value: <sqlx::Sqlite as sqlx::Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Sqlite>>::decode(value)?;
        serde_json::from_str(&s).map_err(Into::into)
    }
}

/// Represents a message that is stored in the vault and displayed in the UI,
/// including the content of the message and metadata like sender and timestamp.
#[derive(TS)]
#[ts(export)]
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct GroupMessage {
    pub id: MessageId,
    pub sender: SentinelIdentity,
    pub group_id: GroupId,
    pub content: GroupMessageContent,
    pub read: bool,
    pub published_at: DateTime<Utc>,
}

impl GroupMessage {
    pub fn new(
        id: MessageId,
        sender: SentinelIdentity,
        group_id: GroupId,
        content: GroupMessageContent,
        read: bool,
        published_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            sender,
            group_id,
            content,
            read,
            published_at,
        }
    }
}

/// A group with its messages, as stored in the vault.
/// Does not include members, since MLS membership is managed in OpenMLS tables.
pub struct GroupWithMessages {
    pub id: GroupId,
    pub display_name: String,
    pub description: String,
    pub messages: Vec<GroupMessage>,
}
