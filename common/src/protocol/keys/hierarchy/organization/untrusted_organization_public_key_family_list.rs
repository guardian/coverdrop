use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::protocol::keys::UntrustedOrganizationPublicKey;

use super::UntrustedOrganizationPublicKeyFamily;

#[derive(Clone, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct UntrustedOrganizationPublicKeyFamilyList(pub Vec<UntrustedOrganizationPublicKeyFamily>);

impl UntrustedOrganizationPublicKeyFamilyList {
    pub fn org_pk_iter(&self) -> impl Iterator<Item = &UntrustedOrganizationPublicKey> {
        self.0.iter().map(|org_pk_family| &org_pk_family.org_pk)
    }
}
