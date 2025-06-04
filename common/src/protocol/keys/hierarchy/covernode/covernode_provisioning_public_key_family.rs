use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::{
    api::models::covernode_id::CoverNodeIdentity,
    protocol::keys::{
        verify_covernode_provisioning_pk, CoverNodeIdPublicKeyFamily,
        CoverNodeIdPublicKeyFamilyList, CoverNodeProvisioningPublicKey, OrganizationPublicKey,
    },
};

use super::untrusted_covernode_provisioning_public_key_family::UntrustedCoverNodeProvisioningPublicKeyFamily;

#[derive(Clone, Debug)]
pub struct CoverNodeProvisioningPublicKeyFamily {
    pub provisioning_pk: CoverNodeProvisioningPublicKey,
    pub covernodes: HashMap<CoverNodeIdentity, CoverNodeIdPublicKeyFamilyList>,
}

impl CoverNodeProvisioningPublicKeyFamily {
    pub fn new(
        provisioning_pk: CoverNodeProvisioningPublicKey,
        covernodes: HashMap<CoverNodeIdentity, CoverNodeIdPublicKeyFamilyList>,
    ) -> Self {
        Self {
            provisioning_pk,
            covernodes,
        }
    }

    pub fn from_untrusted(
        untrusted: UntrustedCoverNodeProvisioningPublicKeyFamily,
        org_pk: &OrganizationPublicKey,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        let covernode_provisioning_pk =
            verify_covernode_provisioning_pk(&untrusted.provisioning_pk, org_pk, now)?;

        let covernodes = untrusted
            .covernodes
            .into_iter()
            // We flat map here, ignoring failures, this is because it's possible that
            // the API call to the published keys happens *just* before a key expires.
            // And we don't want a single key being expired to cause our entire hierarchy
            // to fail to parse
            .map(|(covernode_id, id_pk_family_list)| {
                let id_pk_family_list = CoverNodeIdPublicKeyFamilyList::from_untrusted(
                    id_pk_family_list,
                    &covernode_provisioning_pk,
                    now,
                );

                (covernode_id, id_pk_family_list)
            })
            .collect();

        Ok(Self {
            provisioning_pk: covernode_provisioning_pk,
            covernodes,
        })
    }

    pub fn to_untrusted(&self) -> UntrustedCoverNodeProvisioningPublicKeyFamily {
        UntrustedCoverNodeProvisioningPublicKeyFamily {
            provisioning_pk: self.provisioning_pk.to_untrusted(),
            covernodes: self
                .covernodes
                .iter()
                .map(|(covernode_id, family)| (covernode_id.clone(), family.to_untrusted()))
                .collect(),
        }
    }

    pub fn covernode_iter(
        &self,
    ) -> impl Iterator<Item = (&CoverNodeIdentity, &CoverNodeIdPublicKeyFamily)> {
        self.covernodes
            .iter()
            .flat_map(|(covernode_id, id_pk_family_list)| {
                id_pk_family_list
                    .iter()
                    .map(move |id_pk_family| (covernode_id, id_pk_family))
            })
    }
}
