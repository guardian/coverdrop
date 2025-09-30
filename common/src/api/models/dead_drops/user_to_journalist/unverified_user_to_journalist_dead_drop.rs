use chrono::{DateTime, Utc};
use hex_buffer_serde::Hex;
use serde::{Deserialize, Serialize};

use crate::{
    api::models::dead_drops::DeadDropId,
    crypto::{keys::serde::SignatureHex, Signature},
    epoch::Epoch,
};

use super::{SerializedUserToJournalistDeadDropMessages, UserToJournalistDeadDropSignatureDataV2};

/// A dead drop that has been served from the API but has not yet
/// been verified against the key hierarchy.
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct UnverifiedUserToJournalistDeadDrop {
    pub id: DeadDropId,
    pub created_at: DateTime<Utc>,
    pub data: SerializedUserToJournalistDeadDropMessages,
    #[serde(with = "SignatureHex")]
    pub signature: Signature<UserToJournalistDeadDropSignatureDataV2>,
    pub epoch: Epoch,
}

impl UnverifiedUserToJournalistDeadDrop {
    pub fn signature_data(&self) -> UserToJournalistDeadDropSignatureDataV2 {
        UserToJournalistDeadDropSignatureDataV2::new(&self.data, self.created_at, self.epoch)
    }
}
