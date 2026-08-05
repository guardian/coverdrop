use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    api::models::{journalist_id::JournalistIdentity, sentinel_id::SentinelIdentity},
    protocol::keys::{
        hierarchy::PublishedJournalistIdPublicKeyFamilyList,
        UntrustedJournalistProvisioningPublicKey, UntrustedSentinelIdPublicKeyList,
    },
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UntrustedJournalistProvisioningPublicKeyFamily {
    pub provisioning_pk: UntrustedJournalistProvisioningPublicKey,
    pub journalists: HashMap<JournalistIdentity, PublishedJournalistIdPublicKeyFamilyList>,
    pub sentinel: HashMap<SentinelIdentity, UntrustedSentinelIdPublicKeyList>,
}
