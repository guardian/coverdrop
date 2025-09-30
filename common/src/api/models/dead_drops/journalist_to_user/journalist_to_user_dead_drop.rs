use chrono::{DateTime, Utc};

use crate::api::models::dead_drops::DeadDropId;

use super::journalist_to_user_dead_drop_messages::JournalistToUserDeadDropMessages;

/// A journalist to user dead drop that has been sent from the API and has been verified against
/// a trusted/verified public key hierarchy
pub struct JournalistToUserDeadDrop {
    pub id: DeadDropId,
    pub created_at: DateTime<Utc>,
    pub data: JournalistToUserDeadDropMessages,
}

impl JournalistToUserDeadDrop {
    pub fn new(
        id: DeadDropId,
        created_at: DateTime<Utc>,
        data: JournalistToUserDeadDropMessages,
    ) -> Self {
        Self {
            id,
            created_at,
            data,
        }
    }
}
