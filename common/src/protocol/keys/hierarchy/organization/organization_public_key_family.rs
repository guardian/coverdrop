use chrono::{DateTime, Utc};

use crate::{
    backup::roles::{BackupId, BackupMsg},
    protocol::{
        keys::{
            verify_organization_pk, AnchorOrganizationPublicKey, BackupIdPublicKeyFamilyList,
            CoverNodeProvisioningPublicKeyFamilyList, IdentityPublicKeyFamilyList,
            JournalistProvisioningPublicKeyFamilyList, OrganizationPublicKey,
        },
        roles::Organization,
    },
};

use super::UntrustedOrganizationPublicKeyFamily;

#[derive(Debug)]
pub struct OrganizationPublicKeyFamily {
    pub org_pk: OrganizationPublicKey,
    pub covernodes: CoverNodeProvisioningPublicKeyFamilyList,
    pub journalists: JournalistProvisioningPublicKeyFamilyList,
    pub backups: Option<BackupIdPublicKeyFamilyList>,
}

impl OrganizationPublicKeyFamily {
    pub fn new(
        org_pk: OrganizationPublicKey,
        covernode_keys: CoverNodeProvisioningPublicKeyFamilyList,
        journalists: JournalistProvisioningPublicKeyFamilyList,
        backups: Option<BackupIdPublicKeyFamilyList>,
    ) -> Self {
        Self {
            org_pk,
            covernodes: covernode_keys,
            journalists,
            backups,
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
        let backups: Option<IdentityPublicKeyFamilyList<Organization, BackupId, BackupMsg>> =
            if let Some(untrusted_backups) = untrusted.backup {
                Some(BackupIdPublicKeyFamilyList::from_untrusted(
                    untrusted_backups,
                    &org_pk,
                    now,
                ))
            } else {
                None
            };

        Ok(Self {
            org_pk,
            covernodes,
            journalists,
            backups,
        })
    }

    pub fn to_untrusted(&self) -> UntrustedOrganizationPublicKeyFamily {
        UntrustedOrganizationPublicKeyFamily {
            org_pk: self.org_pk.to_untrusted(),
            covernodes: self.covernodes.to_untrusted(),
            journalists: self.journalists.to_untrusted(),
            backup: self.backups.as_ref().map(|backups| backups.to_untrusted()),
        }
    }
}
