use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::protocol::keys::{
    UntrustedCoverNodeProvisioningPublicKeyFamilyList, UntrustedJournalistPublicKeyHierarchy,
    UntrustedOrganizationPublicKey,
};

#[derive(Clone, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct UntrustedOrganizationPublicKeyFamily {
    #[ts(type = "unknown")]
    pub org_pk: UntrustedOrganizationPublicKey,
    #[ts(type = "unknown")]
    pub covernodes: UntrustedCoverNodeProvisioningPublicKeyFamilyList,
    #[ts(type = "unknown")]
    pub journalists: UntrustedJournalistPublicKeyHierarchy,
}
