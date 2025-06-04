use chrono::{DateTime, Utc};

use crate::{
    api::models::covernode_id::CoverNodeIdentity,
    protocol::keys::{CoverNodeIdPublicKeyFamily, OrganizationPublicKey},
};

use super::{
    CoverNodeProvisioningPublicKeyFamily, UntrustedCoverNodeProvisioningPublicKeyFamilyList,
};

#[derive(Clone, Debug)]
pub struct CoverNodeProvisioningPublicKeyFamilyList(Vec<CoverNodeProvisioningPublicKeyFamily>);

impl CoverNodeProvisioningPublicKeyFamilyList {
    pub fn new(pks: Vec<CoverNodeProvisioningPublicKeyFamily>) -> Self {
        Self(pks)
    }

    pub fn empty() -> Self {
        Self(Vec::new())
    }

    pub fn from_untrusted(
        untrusted: UntrustedCoverNodeProvisioningPublicKeyFamilyList,
        org_pk: &OrganizationPublicKey,
        now: DateTime<Utc>,
    ) -> Self {
        let keys = untrusted
            .0
            .into_iter()
            // We flat map here, ignoring failures, this is because it's possible that
            // the API call to the published keys happens *just* before a key expires.
            // And we don't want a single key being expired to cause our entire hierarchy
            // to fail to verify
            .flat_map(|provisioning_key_family| {
                CoverNodeProvisioningPublicKeyFamily::from_untrusted(
                    provisioning_key_family,
                    org_pk,
                    now,
                )
            })
            .collect();

        Self(keys)
    }

    pub fn to_untrusted(&self) -> UntrustedCoverNodeProvisioningPublicKeyFamilyList {
        let covernode_provisioning_pk_families = self
            .0
            .iter()
            .map(|covernode_provisioning_pk_family| covernode_provisioning_pk_family.to_untrusted())
            .collect();

        UntrustedCoverNodeProvisioningPublicKeyFamilyList(covernode_provisioning_pk_families)
    }

    pub fn iter(&self) -> impl Iterator<Item = &CoverNodeProvisioningPublicKeyFamily> {
        self.0.iter()
    }

    pub fn covernode_pk_family_iter(
        &self,
    ) -> impl Iterator<Item = (&CoverNodeIdentity, &CoverNodeIdPublicKeyFamily)> {
        self.0
            .iter()
            .flat_map(|provisioning_family| provisioning_family.covernode_iter())
    }

    pub fn insert(&mut self, pk_family: CoverNodeProvisioningPublicKeyFamily) {
        self.0.push(pk_family)
    }
}
