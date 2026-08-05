use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::api::models::journalist_id::JournalistIdentity;
use crate::api::models::sentinel_id::SentinelIdentity;
use crate::protocol::keys::{
    verify_journalist_provisioning_pk, JournalistIdPublicKeyFamily,
    JournalistIdPublicKeyFamilyList, JournalistProvisioningPublicKey, OrganizationPublicKey,
    SentinelIdPublicKey, SentinelIdPublicKeyList,
};

use super::UntrustedJournalistProvisioningPublicKeyFamily;

#[derive(Clone, Debug)]
pub struct JournalistProvisioningPublicKeyFamily {
    pub provisioning_pk: JournalistProvisioningPublicKey,
    pub journalists: HashMap<JournalistIdentity, JournalistIdPublicKeyFamilyList>,
    pub sentinel: HashMap<SentinelIdentity, SentinelIdPublicKeyList>,
}

impl JournalistProvisioningPublicKeyFamily {
    pub fn new(
        provisioning_pk: JournalistProvisioningPublicKey,
        journalists: HashMap<JournalistIdentity, JournalistIdPublicKeyFamilyList>,
        sentinel: HashMap<SentinelIdentity, SentinelIdPublicKeyList>,
    ) -> Self {
        Self {
            provisioning_pk,
            journalists,
            sentinel,
        }
    }

    pub fn from_untrusted(
        untrusted: UntrustedJournalistProvisioningPublicKeyFamily,
        org_pk: &OrganizationPublicKey,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        let journalist_provisioning_pk =
            verify_journalist_provisioning_pk(&untrusted.provisioning_pk, org_pk, now)?;

        let journalists = untrusted
            .journalists
            .into_iter()
            // We flat map here, ignoring failures, this is because it's possible that
            // the API call to the published keys happens *just* before a key expires.
            // And we don't want a single key being expired to cause our entire hierarchy
            // to fail to verify
            .map(|(journalist_id, untrusted_id_pk_family_list)| {
                let id_pk_family = JournalistIdPublicKeyFamilyList::from_untrusted(
                    untrusted_id_pk_family_list,
                    &journalist_provisioning_pk,
                    now,
                );

                (journalist_id, id_pk_family)
            })
            .collect();

        let sentinel = untrusted
            .sentinel
            .into_iter()
            .map(|(sentinel_id, untrusted_id_pk_list)| {
                let id_pk_list = SentinelIdPublicKeyList::from_untrusted(
                    untrusted_id_pk_list,
                    &journalist_provisioning_pk,
                    now,
                );

                (sentinel_id, id_pk_list)
            })
            .collect();

        Ok(Self {
            provisioning_pk: journalist_provisioning_pk,
            journalists,
            sentinel,
        })
    }

    pub fn to_untrusted(&self) -> UntrustedJournalistProvisioningPublicKeyFamily {
        UntrustedJournalistProvisioningPublicKeyFamily {
            provisioning_pk: self.provisioning_pk.to_untrusted(),
            journalists: self
                .journalists
                .iter()
                .map(|(journalist_id, family)| (journalist_id.clone(), family.to_untrusted()))
                .collect(),
            sentinel: self
                .sentinel
                .iter()
                .map(|(sentinel_id, id_pk_list)| (sentinel_id.clone(), id_pk_list.to_untrusted()))
                .collect(),
        }
    }

    pub fn journalist_iter(
        &self,
    ) -> impl Iterator<Item = (&JournalistIdentity, &JournalistIdPublicKeyFamily)> {
        self.journalists
            .iter()
            .flat_map(|(journalist_id, pk_family_list)| {
                pk_family_list
                    .iter()
                    .map(move |pk_family| (journalist_id, pk_family))
            })
    }

    pub fn sentinel_iter(&self) -> impl Iterator<Item = (&SentinelIdentity, &SentinelIdPublicKey)> {
        self.sentinel
            .iter()
            .flat_map(|(sentinel_id, pk_list)| pk_list.iter().map(move |pk| (sentinel_id, pk)))
    }
}
