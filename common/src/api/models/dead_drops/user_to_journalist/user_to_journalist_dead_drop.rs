use chrono::{DateTime, Utc};

use crate::{api::models::dead_drops::DeadDropId, crypto::Signature, epoch::Epoch};

use super::{UserToJournalistDeadDropMessages, UserToJournalistDeadDropSignatureDataV2};

/// A user to journalist dead drop that has been sent from the API and has been verified against
/// a trusted/verified public key hierarchy
pub struct UserToJournalistDeadDrop {
    pub id: DeadDropId,
    pub created_at: DateTime<Utc>,
    pub data: UserToJournalistDeadDropMessages,
    pub signature: Signature<UserToJournalistDeadDropSignatureDataV2>,
    pub epoch: Epoch,
}

impl UserToJournalistDeadDrop {
    pub fn new(
        id: DeadDropId,
        created_at: DateTime<Utc>,
        data: UserToJournalistDeadDropMessages,
        signature: Signature<UserToJournalistDeadDropSignatureDataV2>,
        epoch: Epoch,
    ) -> Self {
        Self {
            id,
            created_at,
            data,
            signature,
            epoch,
        }
    }
}
