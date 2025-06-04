use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    api::models::journalist_id::JournalistIdentity,
    protocol::keys::{
        hierarchy::PublishedJournalistIdPublicKeyFamilyList,
        UntrustedJournalistProvisioningPublicKey,
    },
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UntrustedJournalistProvisioningPublicKeyFamily {
    pub provisioning_pk: UntrustedJournalistProvisioningPublicKey,
    pub journalists: HashMap<JournalistIdentity, PublishedJournalistIdPublicKeyFamilyList>,
}
