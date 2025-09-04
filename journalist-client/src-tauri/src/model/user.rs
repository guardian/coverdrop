use common::protocol::roles::User as UserRole;
use common::{
    client::mailbox::mailbox_message, crypto::keys::encryption::PublicEncryptionKey,
    generators::NameGenerator,
};
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
#[serde(rename_all = "camelCase", tag = "type")]
#[ts(export)]
pub struct User {
    user_pk: String,
    status: UserStatus,
    display_name: String,
    alias: Option<String>,
    description: Option<String>,
    marked_as_unread: bool,
}

impl User {
    pub fn new(
        name_generator: &NameGenerator,
        user_pk: PublicEncryptionKey<UserRole>,
        status: UserStatus,
        alias: Option<String>,
        description: Option<String>,
        marked_as_unread: bool,
    ) -> Self {
        let display_name = name_generator.name_from_bytes(user_pk.key.as_bytes(), 2);
        Self {
            user_pk: hex::encode(user_pk.key),
            status,
            display_name,
            alias,
            description,
            marked_as_unread,
        }
    }
}
