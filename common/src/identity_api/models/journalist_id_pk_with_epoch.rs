use serde::{Deserialize, Serialize};

use crate::{epoch::Epoch, protocol::keys::UntrustedJournalistIdPublicKey};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct UntrustedJournalistIdPublicKeyWithEpoch {
    pub epoch: Epoch,
    pub key: UntrustedJournalistIdPublicKey,
}
