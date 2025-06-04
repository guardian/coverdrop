use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    api::models::covernode_id::CoverNodeIdentity,
    protocol::keys::{
        PublishedCoverNodeIdPublicKeyFamilyList, UntrustedCoverNodeProvisioningPublicKey,
    },
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UntrustedCoverNodeProvisioningPublicKeyFamily {
    pub provisioning_pk: UntrustedCoverNodeProvisioningPublicKey,
    pub covernodes: HashMap<CoverNodeIdentity, PublishedCoverNodeIdPublicKeyFamilyList>,
}
