use chrono::{DateTime, Utc};

use crate::protocol::keys::{
    verify_organization_pk, AnchorOrganizationPublicKey, CoverNodeProvisioningPublicKeyFamilyList,
    JournalistProvisioningPublicKeyFamilyList, OrganizationPublicKey,
};

use super::UntrustedOrganizationPublicKeyFamily;

#[derive(Debug)]
pub struct OrganizationPublicKeyFamily {
    pub org_pk: OrganizationPublicKey,
    pub covernodes: CoverNodeProvisioningPublicKeyFamilyList,
    pub journalists: JournalistProvisioningPublicKeyFamilyList,
}

impl OrganizationPublicKeyFamily {
    pub fn new(
        org_pk: OrganizationPublicKey,
        covernode_keys: CoverNodeProvisioningPublicKeyFamilyList,
        journalists: JournalistProvisioningPublicKeyFamilyList,
    ) -> Self {
        Self {
            org_pk,
            covernodes: covernode_keys,
            journalists,
        }
    }

    pub fn from_untrusted(
        untrusted: UntrustedOrganizationPublicKeyFamily,
        anchor_org_pks: &[AnchorOrganizationPublicKey],
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        let Some(org_pk) = anchor_org_pks.iter().find_map(|anchor_org_pk| {
            verify_organization_pk(&untrusted.org_pk, anchor_org_pk, now).ok()
        }) else {
            anyhow::bail!("Published org_pk not found in list of trusted org_pks");
        };

        let covernodes = CoverNodeProvisioningPublicKeyFamilyList::from_untrusted(
            untrusted.covernodes,
            &org_pk,
            now,
        );
        let journalists = JournalistProvisioningPublicKeyFamilyList::from_untrusted(
            untrusted.journalists,
            &org_pk,
            now,
        );

        Ok(Self {
            org_pk,
            covernodes,
            journalists,
        })
    }

    pub fn to_untrusted(&self) -> UntrustedOrganizationPublicKeyFamily {
        UntrustedOrganizationPublicKeyFamily {
            org_pk: self.org_pk.to_untrusted(),
            covernodes: self.covernodes.to_untrusted(),
            journalists: self.journalists.to_untrusted(),
        }
    }
}
