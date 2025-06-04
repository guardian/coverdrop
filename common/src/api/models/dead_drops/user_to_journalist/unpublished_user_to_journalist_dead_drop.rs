use chrono::{DateTime, Utc};
use hex_buffer_serde::Hex;
use serde::{Deserialize, Serialize};

use crate::{
    crypto::{keys::serde::SignatureHex, Signature},
    epoch::Epoch,
};

use super::{
    SerializedUserToJournalistDeadDropMessages, UserToJournalistDeadDropCertificateDataV1,
    UserToJournalistDeadDropSignatureDataV2,
};

// This is a dead drop that has been emitted by the CoverNode
// but has yet to be accepted by the API so it doesn't have an ID
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct UnpublishedUserToJournalistDeadDrop {
    pub data: SerializedUserToJournalistDeadDropMessages,
    // Certificate: deprecated since it doesn't sign over the created_at time
    #[serde(with = "SignatureHex")]
    pub cert: Signature<UserToJournalistDeadDropCertificateDataV1>,
    #[serde(with = "SignatureHex")]
    pub signature: Signature<UserToJournalistDeadDropSignatureDataV2>,
    pub created_at: DateTime<Utc>,
    pub epoch: Epoch,
}

impl UnpublishedUserToJournalistDeadDrop {
    pub fn new(
        data: SerializedUserToJournalistDeadDropMessages,
        cert: Signature<UserToJournalistDeadDropCertificateDataV1>,
        signature: Signature<UserToJournalistDeadDropSignatureDataV2>,
        created_at: DateTime<Utc>,
        epoch: Epoch,
    ) -> Self {
        Self {
            data,
            cert,
            signature,
            created_at,
            epoch,
        }
    }
}
