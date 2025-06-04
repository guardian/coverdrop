use chrono::{DateTime, Utc};

use crate::{
    api::models::journalist_id::JournalistIdentity,
    protocol::keys::{
        JournalistIdPublicKeyFamily, JournalistProvisioningPublicKey, OrganizationPublicKey,
    },
};

use super::{JournalistProvisioningPublicKeyFamily, UntrustedJournalistPublicKeyHierarchy};

#[derive(Clone, Debug)]
pub struct JournalistProvisioningPublicKeyFamilyList(Vec<JournalistProvisioningPublicKeyFamily>);

impl JournalistProvisioningPublicKeyFamilyList {
    pub fn new(pks: Vec<JournalistProvisioningPublicKeyFamily>) -> Self {
        Self(pks)
    }

    pub fn empty() -> Self {
        Self(Vec::new())
    }

    pub fn from_untrusted(
        untrusted: UntrustedJournalistPublicKeyHierarchy,
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
                JournalistProvisioningPublicKeyFamily::from_untrusted(
                    provisioning_key_family,
                    org_pk,
                    now,
                )
            })
            .collect();

        Self(keys)
    }

    pub fn to_untrusted(&self) -> UntrustedJournalistPublicKeyHierarchy {
        let journalist_pk_families = self
            .0
            .iter()
            .map(|journalist_pk_family| journalist_pk_family.to_untrusted())
            .collect();

        UntrustedJournalistPublicKeyHierarchy(journalist_pk_families)
    }

    pub fn journalist_provisioning_pk_iter(
        &self,
    ) -> impl Iterator<Item = &JournalistProvisioningPublicKey> {
        self.0
            .iter()
            .map(|provisioning_family| &provisioning_family.provisioning_pk)
    }

    pub fn journalist_pk_family_iter(
        &self,
    ) -> impl Iterator<Item = (&JournalistIdentity, &JournalistIdPublicKeyFamily)> {
        self.0
            .iter()
            .flat_map(|provisioning_family| provisioning_family.journalist_iter())
    }

    pub fn insert(&mut self, provisioning_pk_family: JournalistProvisioningPublicKeyFamily) {
        self.0.push(provisioning_pk_family)
    }
}
