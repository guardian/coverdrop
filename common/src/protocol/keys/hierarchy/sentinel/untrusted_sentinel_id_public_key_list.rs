use serde::{Deserialize, Serialize};

use crate::protocol::keys::UntrustedSentinelIdPublicKey;

/// The untrusted representation of a list of sentinel identity public keys.
/// Must be verified by transforming to [`SentinelIdPublicKeyList`] before use.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent, deny_unknown_fields)]
pub struct UntrustedSentinelIdPublicKeyList(pub Vec<UntrustedSentinelIdPublicKey>);

impl UntrustedSentinelIdPublicKeyList {
    pub fn new(keys: Vec<UntrustedSentinelIdPublicKey>) -> Self {
        Self(keys)
    }
}

impl IntoIterator for UntrustedSentinelIdPublicKeyList {
    type Item = UntrustedSentinelIdPublicKey;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
