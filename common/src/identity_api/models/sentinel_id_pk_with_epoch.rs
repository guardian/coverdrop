use serde::{Deserialize, Serialize};

use crate::{epoch::Epoch, protocol::keys::UntrustedSentinelIdPublicKey};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct UntrustedSentinelIdPublicKeyWithEpoch {
    pub epoch: Epoch,
    pub key: UntrustedSentinelIdPublicKey,
}
