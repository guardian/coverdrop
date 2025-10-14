use chrono::{DateTime, Utc};

use super::UntrustedBackupPublicKeyFamilyList;
use crate::protocol::keys::{BackupIdPublicKeyFamilyList, OrganizationPublicKey};

#[derive(Clone, Debug)]
pub struct BackupPublicKeyFamilyList(Vec<BackupIdPublicKeyFamilyList>);

impl BackupPublicKeyFamilyList {
    pub fn new(pks: Vec<BackupIdPublicKeyFamilyList>) -> Self {
        Self(pks)
    }

    pub fn empty() -> Self {
        Self(Vec::new())
    }

    pub fn from_untrusted(
        untrusted: UntrustedBackupPublicKeyFamilyList,
        org_pk: &OrganizationPublicKey,
        now: DateTime<Utc>,
    ) -> Self {
        let keys = untrusted
            .0
            .into_iter()
            .map(|backup_pk_family| {
                BackupIdPublicKeyFamilyList::from_untrusted(backup_pk_family, org_pk, now)
            })
            .collect::<Vec<_>>();
        BackupPublicKeyFamilyList(keys)
    }

    pub fn to_untrusted(&self) -> UntrustedBackupPublicKeyFamilyList {
        let backup_pk_families = self
            .0
            .iter()
            .map(|backup_pk_family| backup_pk_family.to_untrusted())
            .collect();

        UntrustedBackupPublicKeyFamilyList(backup_pk_families)
    }

    pub fn insert(&mut self, backup_id_public_key_family_list: BackupIdPublicKeyFamilyList) {
        self.0.push(backup_id_public_key_family_list)
    }
}
