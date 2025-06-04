use chrono::{DateTime, Utc};
use common::protocol::keys::{
    CoverNodeMessagingKeyPair, UnregisteredCoverNodeIdKeyPair, UntrustedCoverNodeIdKeyPair,
    UntrustedCoverNodeMessagingKeyPair, UntrustedUnregisteredCoverNodeIdKeyPair,
};

pub struct CandidateKeyPairWithCreatedAt<T> {
    pub key_pair: T,
    pub created_at: DateTime<Utc>,
}

impl<T> CandidateKeyPairWithCreatedAt<T> {
    pub fn new(key_pair: T, created_at: DateTime<Utc>) -> Self {
        Self {
            key_pair,
            created_at,
        }
    }
}

pub type UntrustedCandidateCoverNodeMessagingKeyPairWithCreatedAt =
    CandidateKeyPairWithCreatedAt<UntrustedCoverNodeMessagingKeyPair>;
pub type CandidateCoverNodeMessagingKeyPairWithCreatedAt =
    CandidateKeyPairWithCreatedAt<CoverNodeMessagingKeyPair>;

pub type UntrustedCandidateCoverNodeIdKeyPairWithCreatedAt =
    CandidateKeyPairWithCreatedAt<UntrustedUnregisteredCoverNodeIdKeyPair>;
pub type CandidateUnregisteredCoverNodeIdKeyPairWithCreatedAt =
    CandidateKeyPairWithCreatedAt<UnregisteredCoverNodeIdKeyPair>;

pub type UntrustedCoverNodeIdKeyPairWithCreatedAt =
    CandidateKeyPairWithCreatedAt<UntrustedCoverNodeIdKeyPair>;
