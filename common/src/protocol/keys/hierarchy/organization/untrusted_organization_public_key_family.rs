use serde::{Deserialize, Serialize};

use crate::protocol::keys::{
    UntrustedCoverNodeProvisioningPublicKeyFamilyList, UntrustedJournalistPublicKeyHierarchy,
    UntrustedOrganizationPublicKey,
};

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UntrustedOrganizationPublicKeyFamily {
    pub org_pk: UntrustedOrganizationPublicKey,
    pub covernodes: UntrustedCoverNodeProvisioningPublicKeyFamilyList,
    pub journalists: UntrustedJournalistPublicKeyHierarchy,
}
