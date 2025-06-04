use chrono::{DateTime, Utc};
use openssl::sha::Sha256;

use crate::{crypto::Signable, epoch::Epoch};

use super::SerializedUserToJournalistDeadDropMessages;

/// A representation of the data required to sign/verify a user to
/// journalist dead drop.
#[derive(Debug)]
pub struct UserToJournalistDeadDropSignatureDataV2(pub [u8; 32]);

impl UserToJournalistDeadDropSignatureDataV2 {
    pub fn new(
        serialized_dead_drop_messages: &SerializedUserToJournalistDeadDropMessages,
        created_at: DateTime<Utc>,
        epoch: Epoch,
    ) -> Self {
        let mut hasher = Sha256::new();

        hasher.update(serialized_dead_drop_messages.as_bytes());
        hasher.update(&created_at.timestamp().to_be_bytes()[..]);
        hasher.update(&epoch.to_be_bytes()[..]);

        let hash = hasher.finish();

        Self(hash)
    }
}

impl Signable for UserToJournalistDeadDropSignatureDataV2 {
    fn as_signable_bytes(&self) -> &[u8] {
        &self.0[..]
    }
}
