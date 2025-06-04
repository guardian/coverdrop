use serde::{Deserialize, Serialize};

use super::UntrustedCoverNodeProvisioningPublicKeyFamily;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UntrustedCoverNodeProvisioningPublicKeyFamilyList(
    pub Vec<UntrustedCoverNodeProvisioningPublicKeyFamily>,
);
