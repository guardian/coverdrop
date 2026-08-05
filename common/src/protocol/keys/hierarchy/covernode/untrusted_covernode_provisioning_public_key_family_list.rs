use serde::{Deserialize, Serialize};

use super::UntrustedCoverNodeProvisioningPublicKeyFamily;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UntrustedCoverNodeProvisioningPublicKeyFamilyList(
    pub Vec<UntrustedCoverNodeProvisioningPublicKeyFamily>,
);
