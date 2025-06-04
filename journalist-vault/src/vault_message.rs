use chrono::{DateTime, Utc};
use common::{
    client::mailbox::mailbox_message::UserStatus, protocol::keys::UserPublicKey,
    FixedSizeMessageText,
};

#[derive(Clone)]
pub struct U2JMessage {
    pub id: i64,
    pub user_pk: UserPublicKey,
    // TODO: should be in a new user type
    pub user_status: UserStatus,
    pub message: FixedSizeMessageText,
    pub received_at: DateTime<Utc>,
    pub read: bool,
    pub user_alias: Option<String>,
    pub user_description: Option<String>,
}

#[derive(Clone)]
pub struct J2UMessage {
    pub id: i64,
    pub user_pk: UserPublicKey,
    // TODO: should be in a new user type
    pub user_status: UserStatus,
    pub message: FixedSizeMessageText,
    pub is_sent: bool,
    pub sent_at: DateTime<Utc>,
    pub user_alias: Option<String>,
    pub user_description: Option<String>,
}

impl U2JMessage {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: i64,
        from: UserPublicKey,
        user_status: UserStatus,
        message: &FixedSizeMessageText,
        received_at: DateTime<Utc>,
        read: bool,
        user_alias: Option<String>,
        user_description: Option<String>,
    ) -> Self {
        Self {
            id,
            user_pk: from,
            user_status,
            message: message.clone(),
            received_at,
            read,
            user_alias,
            user_description,
        }
    }
}

impl J2UMessage {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: i64,
        to: UserPublicKey,
        user_status: UserStatus,
        message: &FixedSizeMessageText,
        is_sent: bool,
        sent_at: DateTime<Utc>,
        user_alias: Option<String>,
        user_description: Option<String>,
    ) -> Self {
        Self {
            id,
            user_pk: to,
            user_status,
            message: message.clone(),
            is_sent,
            sent_at,
            user_alias,
            user_description,
        }
    }
}

#[derive(Clone)]
pub enum VaultMessage {
    U2J(U2JMessage),
    J2U(J2UMessage),
}
