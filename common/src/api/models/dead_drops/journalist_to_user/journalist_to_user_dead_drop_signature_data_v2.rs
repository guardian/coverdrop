use chrono::{DateTime, Utc};
use openssl::sha::Sha256;

use crate::crypto::Signable;

use super::SerializedJournalistToUserDeadDropMessages;

/// A representation of the data required to sign/verify a journalist to
/// user dead drop.
#[derive(Debug)]
pub struct JournalistToUserDeadDropSignatureDataV2(pub [u8; 32]);

impl JournalistToUserDeadDropSignatureDataV2 {
    pub fn new(
        serialized_dead_drop_messages: &SerializedJournalistToUserDeadDropMessages,
        created_at: DateTime<Utc>,
    ) -> Self {
        let mut hasher = Sha256::new();

        hasher.update(serialized_dead_drop_messages.as_signable_bytes());
        hasher.update(&created_at.timestamp().to_be_bytes()[..]);

        let hash = hasher.finish();

        Self(hash)
    }
}

impl Signable for JournalistToUserDeadDropSignatureDataV2 {
    fn as_signable_bytes(&self) -> &[u8] {
        &self.0[..]
    }
}
