use chrono::{DateTime, Utc};

use crate::{api::models::dead_drops::DeadDropId, crypto::Signature, epoch::Epoch};

use super::{
    UserToJournalistDeadDropCertificateDataV1, UserToJournalistDeadDropMessages,
    UserToJournalistDeadDropSignatureDataV2,
};

/// A user to journalist dead drop that has been sent from the API and has been verified against
/// a trusted/verified public key hierarchy
pub struct UserToJournalistDeadDrop {
    pub id: DeadDropId,
    pub created_at: DateTime<Utc>,
    pub data: UserToJournalistDeadDropMessages,
    // Certificate: being phased out since it does not contain the created_at
    // timestamp as part of the signature
    pub cert: Signature<UserToJournalistDeadDropCertificateDataV1>,

    // Signature: newer, contains the timestamp
    pub signature: Signature<UserToJournalistDeadDropSignatureDataV2>,
    pub epoch: Epoch,
}

impl UserToJournalistDeadDrop {
    pub fn new(
        id: DeadDropId,
        created_at: DateTime<Utc>,
        data: UserToJournalistDeadDropMessages,
        cert: Signature<UserToJournalistDeadDropCertificateDataV1>,
        signature: Signature<UserToJournalistDeadDropSignatureDataV2>,
        epoch: Epoch,
    ) -> Self {
        Self {
            id,
            created_at,
            data,
            cert,
            signature,
            epoch,
        }
    }
}
