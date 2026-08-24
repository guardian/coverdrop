use chrono::{DateTime, Utc};

use crate::api::models::dead_drops::DeadDropId;

use super::user_to_journalist_message::UserToJournalistMessage;

/// Wrapper for [UserToJournalistMessage] including the ID and created_at timestamp
/// of the U2J dead drop which contained it.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct U2JMessageWithMetadata {
    pub u2j_message: UserToJournalistMessage,
    pub unsigned_dead_drop_id: DeadDropId,
    pub dead_drop_created_at: DateTime<Utc>,
}
