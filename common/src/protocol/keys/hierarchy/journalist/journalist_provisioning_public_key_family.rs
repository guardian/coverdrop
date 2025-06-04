use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::api::models::journalist_id::JournalistIdentity;
use crate::protocol::keys::{
    verify_journalist_provisioning_pk, JournalistIdPublicKeyFamily,
    JournalistIdPublicKeyFamilyList, JournalistProvisioningPublicKey, OrganizationPublicKey,
};

use super::UntrustedJournalistProvisioningPublicKeyFamily;

#[derive(Clone, Debug)]
pub struct JournalistProvisioningPublicKeyFamily {
    pub provisioning_pk: JournalistProvisioningPublicKey,
    pub journalists: HashMap<JournalistIdentity, JournalistIdPublicKeyFamilyList>,
}

impl JournalistProvisioningPublicKeyFamily {
    pub fn new(
        provisioning_pk: JournalistProvisioningPublicKey,
        journalists: HashMap<JournalistIdentity, JournalistIdPublicKeyFamilyList>,
    ) -> Self {
        Self {
            provisioning_pk,
            journalists,
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

        Ok(Self {
            provisioning_pk: journalist_provisioning_pk,
            journalists,
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
}
