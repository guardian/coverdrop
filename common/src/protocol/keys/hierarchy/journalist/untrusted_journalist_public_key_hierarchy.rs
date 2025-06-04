use serde::{Deserialize, Serialize};

use super::UntrustedJournalistProvisioningPublicKeyFamily;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UntrustedJournalistPublicKeyHierarchy(
    pub Vec<UntrustedJournalistProvisioningPublicKeyFamily>,
);
