use chrono::{DateTime, Utc};
use hex_buffer_serde::Hex;
use serde::{Deserialize, Serialize};

use crate::crypto::{keys::serde::SignatureHex, Signature};

use super::{
    journalist_to_user_dead_drop_signature_data_v2::JournalistToUserDeadDropSignatureDataV2,
    SerializedJournalistToUserDeadDropMessages,
};

/// A signed dead drop that has been sent by a CoverNode but has not yet
/// been accepted into the API.
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct UnpublishedJournalistToUserDeadDrop {
    pub data: SerializedJournalistToUserDeadDropMessages,
    pub created_at: DateTime<Utc>,
    #[serde(with = "SignatureHex")]
    pub signature: Signature<JournalistToUserDeadDropSignatureDataV2>,
}

impl UnpublishedJournalistToUserDeadDrop {
    pub fn new(
        data: SerializedJournalistToUserDeadDropMessages,
        created_at: DateTime<Utc>,
        signature: Signature<JournalistToUserDeadDropSignatureDataV2>,
    ) -> Self {
        Self {
            data,
            created_at,
            signature,
        }
    }
}
