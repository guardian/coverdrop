use serde::{Deserialize, Serialize};

use crate::protocol::keys::{
    UntrustedAnchorOrganizationPublicKey, UntrustedCoverNodeProvisioningPublicKey,
    UntrustedJournalistProvisioningPublicKey,
};

#[derive(Serialize, Deserialize)]
pub struct IdentityApiPublicKeys {
    pub anchor_org_pks: Vec<UntrustedAnchorOrganizationPublicKey>,
    pub covernode_provisioning_pk: Option<UntrustedCoverNodeProvisioningPublicKey>,
    pub journalist_provisioning_pk: Option<UntrustedJournalistProvisioningPublicKey>,
}

impl IdentityApiPublicKeys {
    pub fn new(
        anchor_org_pks: Vec<UntrustedAnchorOrganizationPublicKey>,
        covernode_provisioning_pk: Option<UntrustedCoverNodeProvisioningPublicKey>,
        journalist_provisioning_pk: Option<UntrustedJournalistProvisioningPublicKey>,
    ) -> Self {
        Self {
            anchor_org_pks,
            covernode_provisioning_pk,
            journalist_provisioning_pk,
        }
    }
}
