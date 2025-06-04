use serde::{Deserialize, Serialize};

use crate::{epoch::Epoch, protocol::keys::UntrustedCoverNodeIdPublicKey};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct UntrustedCoverNodeIdPublicKeyWithEpoch {
    pub epoch: Epoch,
    pub key: UntrustedCoverNodeIdPublicKey,
}
