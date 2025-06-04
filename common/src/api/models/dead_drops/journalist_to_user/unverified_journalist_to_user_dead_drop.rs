use chrono::{DateTime, Utc};
use hex_buffer_serde::Hex;
use serde::{Deserialize, Serialize};

use crate::{
    api::models::dead_drops::DeadDropId,
    crypto::{keys::serde::SignatureHex, Signature},
};

use super::{JournalistToUserDeadDropSignatureDataV2, SerializedJournalistToUserDeadDropMessages};

/// A dead drop that has been served from the API but has not yet
/// been verified against the key hierarchy.
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct UnverifiedJournalistToUserDeadDrop {
    pub id: DeadDropId,
    pub created_at: DateTime<Utc>,
    pub data: SerializedJournalistToUserDeadDropMessages,
    #[serde(with = "SignatureHex")]
    pub cert: Signature<SerializedJournalistToUserDeadDropMessages>,
    #[serde(with = "SignatureHex")]
    pub signature: Signature<JournalistToUserDeadDropSignatureDataV2>,
}

impl UnverifiedJournalistToUserDeadDrop {
    pub fn signature_data(&self) -> JournalistToUserDeadDropSignatureDataV2 {
        JournalistToUserDeadDropSignatureDataV2::new(&self.data, self.created_at)
    }
}
