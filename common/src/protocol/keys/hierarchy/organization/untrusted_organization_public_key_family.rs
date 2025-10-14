use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    backup::roles::{BackupId, BackupMsg},
    protocol::{
        keys::{
            UntrustedCoverNodeProvisioningPublicKeyFamilyList,
            UntrustedIdentityPublicKeyFamilyList, UntrustedJournalistPublicKeyHierarchy,
            UntrustedOrganizationPublicKey,
        },
        roles::Organization,
    },
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
    #[ts(type = "unknown")]
    pub backup: Option<UntrustedIdentityPublicKeyFamilyList<Organization, BackupId, BackupMsg>>,
}
