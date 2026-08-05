use chrono::{DateTime, Utc};

use crate::protocol::keys::{JournalistProvisioningPublicKey, SentinelIdPublicKey};

use super::UntrustedSentinelIdPublicKeyList;

/// A verified list of sentinel identity public keys.
///
/// Unlike journalist or covernode identity keys, sentinel identity keys
/// have no child messaging keys.
#[derive(Clone, Debug)]
pub struct SentinelIdPublicKeyList(Vec<SentinelIdPublicKey>);

impl SentinelIdPublicKeyList {
    pub fn new(keys: Vec<SentinelIdPublicKey>) -> Self {
        Self(keys)
    }

    pub fn empty() -> Self {
        Self(vec![])
    }

    pub fn from_untrusted(
        keys: UntrustedSentinelIdPublicKeyList,
        verifying_pk: &JournalistProvisioningPublicKey,
        now: DateTime<Utc>,
    ) -> Self {
        let keys = keys
            .into_iter()
            .flat_map(|untrusted_pk| untrusted_pk.to_trusted(verifying_pk, now))
            .collect();

        Self(keys)
    }

    pub fn to_untrusted(&self) -> UntrustedSentinelIdPublicKeyList {
        UntrustedSentinelIdPublicKeyList::new(self.0.iter().map(|pk| pk.to_untrusted()).collect())
    }

    pub fn iter(&self) -> impl Iterator<Item = &SentinelIdPublicKey> {
        self.0.iter()
    }

    pub fn insert(&mut self, pk: SentinelIdPublicKey) {
        self.0.push(pk);
    }
}
