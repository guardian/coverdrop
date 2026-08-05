use serde::{Deserialize, Serialize};

use super::UntrustedJournalistProvisioningPublicKeyFamily;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UntrustedJournalistPublicKeyHierarchy(
    pub Vec<UntrustedJournalistProvisioningPublicKeyFamily>,
);
