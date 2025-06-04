use chrono::{DateTime, Utc};

use crate::{api::models::dead_drops::DeadDropId, crypto::Signature};

use super::{
    journalist_to_user_dead_drop_messages::JournalistToUserDeadDropMessages,
    SerializedJournalistToUserDeadDropMessages,
};

/// A journalist to user dead drop that has been sent from the API and has been verified against
/// a trusted/verified public key hierarchy
pub struct JournalistToUserDeadDrop {
    pub id: DeadDropId,
    pub created_at: DateTime<Utc>,
    pub data: JournalistToUserDeadDropMessages,
    pub cert: Signature<SerializedJournalistToUserDeadDropMessages>,
}

impl JournalistToUserDeadDrop {
    pub fn new(
        id: DeadDropId,
        created_at: DateTime<Utc>,
        data: JournalistToUserDeadDropMessages,
        cert: Signature<SerializedJournalistToUserDeadDropMessages>,
    ) -> Self {
        Self {
            id,
            created_at,
            data,
            cert,
        }
    }
}
