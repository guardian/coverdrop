use chrono::{DateTime, Utc};

use crate::{
    api::models::{covernode_id::CoverNodeIdentity, journalist_id::JournalistIdentity},
    backup::keys::BackupMsgPublicKey,
    crypto::keys::Ed25519PublicKey,
    protocol::keys::{
        AnchorOrganizationPublicKey, CoverNodeIdKeyPair, CoverNodeIdPublicKey,
        CoverNodeMessagingKeyPair, CoverNodeMessagingPublicKey, CoverNodeProvisioningPublicKey,
        JournalistIdPublicKey, JournalistMessagingPublicKey, JournalistProvisioningPublicKey,
        OrganizationPublicKey, UntrustedCoverNodeIdKeyPair, UntrustedCoverNodeIdPublicKey,
        UntrustedCoverNodeMessagingKeyPair, UntrustedCoverNodeMessagingPublicKey,
        UntrustedCoverNodeProvisioningPublicKey, UntrustedJournalistIdPublicKey,
        UntrustedJournalistMessagingPublicKey, UntrustedJournalistProvisioningPublicKey,
    },
};

use super::{
    organization_public_key_family::OrganizationPublicKeyFamily,
    UntrustedOrganizationPublicKeyFamilyList,
};

/// Represents the entirety of the CoverDrop's public key hierarchy.
///
/// Has many useful functions for taking horizontal slices out of the hierarchy.

#[derive(Debug)]
pub struct OrganizationPublicKeyFamilyList(Vec<OrganizationPublicKeyFamily>);

impl OrganizationPublicKeyFamilyList {
    pub fn new(pks: Vec<OrganizationPublicKeyFamily>) -> Self {
        Self(pks)
    }

    pub fn empty() -> Self {
        Self(Vec::new())
    }

    pub fn from_untrusted(
        untrusted: UntrustedOrganizationPublicKeyFamilyList,
        org_pks: &[AnchorOrganizationPublicKey],
        now: DateTime<Utc>,
    ) -> Self {
        let org_pk_families = untrusted
            .clone()
            .0
            .into_iter()
            .flat_map(
                |org_pk_family| -> anyhow::Result<OrganizationPublicKeyFamily> {
                    OrganizationPublicKeyFamily::from_untrusted(org_pk_family, org_pks, now)
                },
            )
            .collect();

        Self(org_pk_families)
    }

    pub fn to_untrusted(&self) -> UntrustedOrganizationPublicKeyFamilyList {
        UntrustedOrganizationPublicKeyFamilyList(
            self.0
                .iter()
                .map(|org_pk_family| org_pk_family.to_untrusted())
                .collect(),
        )
    }

    //
    // Key accessors:
    //     Used to iterate over types of key, or fetch the latest versions
    //

    pub fn org_pk_iter(&self) -> impl Iterator<Item = &OrganizationPublicKey> {
        self.0.iter().map(|org_pk_family| &org_pk_family.org_pk)
    }

    pub fn latest_org_pk(&self) -> Option<&OrganizationPublicKey> {
        self.org_pk_iter().max_by_key(|id_pk| id_pk.not_valid_after)
    }

    pub fn covernode_provisioning_pk_iter(
        &self,
    ) -> impl Iterator<Item = &CoverNodeProvisioningPublicKey> {
        self.0.iter().flat_map(|org_pk_family| {
            org_pk_family
                .covernodes
                .iter()
                .map(|covernode_key_family| &covernode_key_family.provisioning_pk)
        })
    }

    pub fn latest_covernode_provisioning_pk(&self) -> Option<&CoverNodeProvisioningPublicKey> {
        self.covernode_provisioning_pk_iter()
            .max_by_key(|id_pk| id_pk.not_valid_after)
    }

    pub fn covernode_id_iter(&self) -> impl Iterator<Item = &CoverNodeIdentity> {
        self.0.iter().flat_map(|org_pk_family| {
            org_pk_family
                .covernodes
                .iter()
                .flat_map(|covernode_provisioning_pk_family| {
                    covernode_provisioning_pk_family
                        .covernode_iter()
                        .map(|(covernode_id, _)| covernode_id)
                })
        })
    }

    pub fn covernode_id_pk_iter(
        &self,
    ) -> impl Iterator<Item = (&CoverNodeIdentity, &CoverNodeIdPublicKey)> {
        self.0.iter().flat_map(|org_pk_family| {
            org_pk_family
                .covernodes
                .iter()
                .flat_map(|covernode_provisioning_pk_family| {
                    covernode_provisioning_pk_family.covernode_iter().map(
                        |(covernode_id, covernode_id_family)| {
                            (covernode_id, &covernode_id_family.id_pk)
                        },
                    )
                })
        })
    }

    pub fn covernode_id_pk_iter_for_identity<'a>(
        &'a self,
        covernode_id: &'a CoverNodeIdentity,
    ) -> impl Iterator<Item = &'a CoverNodeIdPublicKey> {
        self.covernode_id_pk_iter().filter_map(move |(id, id_pk)| {
            if id == covernode_id {
                Some(id_pk)
            } else {
                None
            }
        })
    }

    pub fn latest_covernode_id_pk(
        &self,
        covernode_id: &CoverNodeIdentity,
    ) -> Option<&CoverNodeIdPublicKey> {
        self.covernode_id_pk_iter()
            .filter_map(|(id, msg_pk)| {
                if id == covernode_id {
                    Some(msg_pk)
                } else {
                    None
                }
            })
            .max_by_key(|msg_pk| msg_pk.not_valid_after)
    }

    pub fn latest_covernode_id_pk_iter(
        &self,
    ) -> impl Iterator<Item = (&CoverNodeIdentity, &CoverNodeIdPublicKey)> {
        self.covernode_id_iter().flat_map(|covernode_id| {
            self.latest_covernode_id_pk(covernode_id)
                .map(|msg_pk| (covernode_id, msg_pk))
        })
    }

    pub fn verify_covernode_id_key_pairs_batch<'a>(
        &self,
        untrusted_id_pair_iter: impl Iterator<Item = &'a UntrustedCoverNodeIdKeyPair>,
        now: DateTime<Utc>,
    ) -> Vec<CoverNodeIdKeyPair> {
        untrusted_id_pair_iter
            .flat_map(|key_pair| {
                self.covernode_provisioning_pk_iter()
                    .find_map(|covernode_id_pk| key_pair.to_trusted(covernode_id_pk, now).ok())
            })
            .collect()
    }

    pub fn covernode_msg_pk_iter(
        &self,
    ) -> impl Iterator<Item = (&CoverNodeIdentity, &CoverNodeMessagingPublicKey)> {
        self.0.iter().flat_map(|org_pk_family| {
            org_pk_family
                .covernodes
                .iter()
                .flat_map(|covernode_provisioning_pk_family| {
                    covernode_provisioning_pk_family.covernode_iter().flat_map(
                        |(covernode_id, covernode_id_family)| {
                            covernode_id_family
                                .msg_pks
                                .iter()
                                .map(move |msg_pk| (covernode_id, msg_pk))
                        },
                    )
                })
        })
    }

    pub fn verify_covernode_msg_key_pairs_batch<'a>(
        &self,
        covernode_id: &CoverNodeIdentity,
        untrusted_msg_key_pair_iter: impl Iterator<Item = &'a UntrustedCoverNodeMessagingKeyPair>,
        now: DateTime<Utc>,
    ) -> Vec<CoverNodeMessagingKeyPair> {
        untrusted_msg_key_pair_iter
            .flat_map(|key_pair| {
                self.covernode_id_pk_iter_for_identity(covernode_id)
                    .find_map(|covernode_id_pk| key_pair.to_trusted(covernode_id_pk, now).ok())
            })
            .collect()
    }

    pub fn latest_covernode_msg_pk(
        &self,
        covernode_id: &CoverNodeIdentity,
    ) -> Option<&CoverNodeMessagingPublicKey> {
        self.covernode_msg_pk_iter()
            .filter_map(|(id, msg_pk)| {
                if id == covernode_id {
                    Some(msg_pk)
                } else {
                    None
                }
            })
            .max_by_key(|msg_pk| msg_pk.not_valid_after)
    }

    pub fn latest_covernode_msg_pk_iter(
        &self,
    ) -> impl Iterator<Item = (&CoverNodeIdentity, &CoverNodeMessagingPublicKey)> {
        self.covernode_id_iter().flat_map(|covernode_id| {
            self.latest_covernode_msg_pk(covernode_id)
                .map(|msg_pk| (covernode_id, msg_pk))
        })
    }

    pub fn journalist_id_iter(&self) -> impl Iterator<Item = &JournalistIdentity> {
        self.0.iter().flat_map(|org_pk_family| {
            org_pk_family
                .journalists
                .journalist_pk_family_iter()
                .map(|(journalist_id, _)| journalist_id)
        })
    }

    pub fn org_and_journalist_provisioning_pk_iter(
        &self,
    ) -> impl Iterator<Item = (&OrganizationPublicKey, &JournalistProvisioningPublicKey)> {
        self.0.iter().flat_map(|org_pk_family| {
            org_pk_family
                .journalists
                .journalist_provisioning_pk_iter()
                .map(|journalist_provisioning_pk| {
                    (&org_pk_family.org_pk, journalist_provisioning_pk)
                })
        })
    }

    pub fn journalist_provisioning_pk_iter(
        &self,
    ) -> impl Iterator<Item = &JournalistProvisioningPublicKey> {
        self.0
            .iter()
            .flat_map(|org_pk_family| org_pk_family.journalists.journalist_provisioning_pk_iter())
    }

    pub fn latest_journalist_provisioning_pk(&self) -> Option<&JournalistProvisioningPublicKey> {
        self.journalist_provisioning_pk_iter()
            .max_by_key(|id_pk| id_pk.not_valid_after)
    }

    pub fn journalist_id_pk_iter(
        &self,
    ) -> impl Iterator<Item = (&JournalistIdentity, &JournalistIdPublicKey)> {
        self.0.iter().flat_map(|org_pk_family| {
            org_pk_family.journalists.journalist_pk_family_iter().map(
                |(journalist_id, journalist_pk_family)| {
                    (journalist_id, &journalist_pk_family.id_pk)
                },
            )
        })
    }

    pub fn latest_journalist_id_pk(
        &self,
        journalist_id: &JournalistIdentity,
    ) -> Option<&JournalistIdPublicKey> {
        self.journalist_id_pk_iter()
            .filter_map(|(iter_journalist_id, id_pk)| {
                if iter_journalist_id == journalist_id {
                    Some(id_pk)
                } else {
                    None
                }
            })
            .max_by_key(|id_pk| id_pk.not_valid_after)
    }

    pub fn latest_journalist_id_pk_iter(
        &self,
    ) -> impl Iterator<Item = (&JournalistIdentity, &JournalistIdPublicKey)> {
        self.journalist_id_iter().flat_map(|journalist_id| {
            self.latest_journalist_id_pk(journalist_id)
                .map(|id_pk| (journalist_id, id_pk))
        })
    }

    pub fn journalist_msg_pk_iter(
        &self,
    ) -> impl Iterator<Item = (&JournalistIdentity, &JournalistMessagingPublicKey)> {
        self.0.iter().flat_map(|org_pk_family| {
            org_pk_family
                .journalists
                .journalist_pk_family_iter()
                .flat_map(|(journalist_id, journalist_pk_family)| {
                    journalist_pk_family
                        .msg_pks
                        .iter()
                        .map(move |msg_pk| (journalist_id, msg_pk))
                })
        })
    }

    pub fn journalist_id_pk_iter_for_identity<'a>(
        &'a self,
        journalist_id: &'a JournalistIdentity,
    ) -> impl Iterator<Item = &'a JournalistIdPublicKey> {
        self.journalist_id_pk_iter().filter_map(move |(id, id_pk)| {
            if id == journalist_id {
                Some(id_pk)
            } else {
                None
            }
        })
    }

    pub fn journalist_msg_pk_iter_for_identity<'a>(
        &'a self,
        journalist_id: &'a JournalistIdentity,
    ) -> impl Iterator<Item = &'a JournalistMessagingPublicKey> {
        self.journalist_msg_pk_iter()
            .filter_map(move |(id, id_pk)| {
                if id == journalist_id {
                    Some(id_pk)
                } else {
                    None
                }
            })
    }

    pub fn latest_journalist_msg_pk(
        &self,
        journalist_id: &JournalistIdentity,
    ) -> Option<&JournalistMessagingPublicKey> {
        self.journalist_msg_pk_iter()
            .filter_map(|(iter_journalist_id, msg_pk)| {
                if iter_journalist_id == journalist_id {
                    Some(msg_pk)
                } else {
                    None
                }
            })
            .max_by_key(|msg_pk| msg_pk.not_valid_after)
    }

    pub fn latest_journalist_msg_pk_iter(
        &self,
    ) -> impl Iterator<Item = (&JournalistIdentity, &JournalistMessagingPublicKey)> {
        self.journalist_id_iter().flat_map(|journalist_id| {
            self.latest_journalist_msg_pk(journalist_id)
                .map(|msg_pk| (journalist_id, msg_pk))
        })
    }

    pub fn backup_msg_pk_iter(&self) -> impl Iterator<Item = &BackupMsgPublicKey> {
        self.0.iter().flat_map(|org_pk_family| {
            org_pk_family
                .backups
                .iter()
                .flat_map(|backup| backup.msg_pk_iter())
        })
    }

    pub fn latest_backup_msg_pk(&self) -> Option<BackupMsgPublicKey> {
        self.0
            .iter()
            .flat_map(|org_pk_family| {
                org_pk_family
                    .backups
                    .clone()
                    .into_iter()
                    .flat_map(|backup| backup.latest_msg_pk().cloned())
            })
            .max_by_key(|msg_pk| msg_pk.not_valid_after)
    }

    // Getter:
    //    Get various public signing keys using their raw Ed25519 format
    //    these are used when we have been sent a signing key and we want
    //    to verify that it is part of our verified key hierarchy

    pub fn find_org_pk_from_raw_ed25519_pk(
        &self,
        candidate_key: &Ed25519PublicKey,
    ) -> Option<&OrganizationPublicKey> {
        self.org_pk_iter().find(|pk| pk.key == *candidate_key)
    }

    pub fn find_covernode_provisioning_pk_from_raw_ed25519_pk(
        &self,
        candidate_key: &Ed25519PublicKey,
    ) -> Option<&CoverNodeProvisioningPublicKey> {
        self.covernode_provisioning_pk_iter()
            .find(|pk| pk.key == *candidate_key)
    }

    pub fn find_covernode_id_pk_from_raw_ed25519_pk(
        &self,
        candidate_key: &Ed25519PublicKey,
    ) -> Option<(&CoverNodeIdentity, &CoverNodeIdPublicKey)> {
        self.covernode_id_pk_iter()
            .find(|(_, pk)| pk.key == *candidate_key)
    }

    pub fn find_journalist_provisioning_pk_from_raw_ed25519_pk(
        &self,
        candidate_key: &Ed25519PublicKey,
    ) -> Option<&JournalistProvisioningPublicKey> {
        self.journalist_provisioning_pk_iter()
            .find(|pk| pk.key == *candidate_key)
    }

    pub fn find_journalist_id_pk_from_raw_ed25519_pk(
        &self,
        candidate_key: &Ed25519PublicKey,
    ) -> Option<(&JournalistIdentity, &JournalistIdPublicKey)> {
        self.journalist_id_pk_iter()
            .find(|(_, pk)| pk.key == *candidate_key)
    }

    //
    // Verification:
    //    Used when you have a verified key hierarchy and you want to
    //    verify another key, for example one you've loaded off disk.
    //

    pub fn verify_covernode_provisioning_key(
        &self,
        untrusted: UntrustedCoverNodeProvisioningPublicKey,
        now: DateTime<Utc>,
    ) -> Option<CoverNodeProvisioningPublicKey> {
        self.org_pk_iter()
            .find_map(|org_pk| untrusted.to_trusted(org_pk, now).ok())
    }

    pub fn verify_covernode_id_key(
        &self,
        untrusted: UntrustedCoverNodeIdPublicKey,
        now: DateTime<Utc>,
    ) -> Option<CoverNodeIdPublicKey> {
        self.covernode_provisioning_pk_iter()
            .find_map(|covernode_provisioning_pk| {
                untrusted.to_trusted(covernode_provisioning_pk, now).ok()
            })
    }

    pub fn verify_covernode_messaging_key(
        &self,
        untrusted: &UntrustedCoverNodeMessagingPublicKey,
        now: DateTime<Utc>,
    ) -> Option<CoverNodeMessagingPublicKey> {
        self.covernode_id_pk_iter()
            .find_map(|(_, covernode_id_pk)| untrusted.to_trusted(covernode_id_pk, now).ok())
    }

    pub fn verify_journalist_provisioning_key(
        &self,
        untrusted: UntrustedJournalistProvisioningPublicKey,
        now: DateTime<Utc>,
    ) -> Option<JournalistProvisioningPublicKey> {
        self.org_pk_iter()
            .find_map(|org_pk| untrusted.to_trusted(org_pk, now).ok())
    }

    pub fn verify_journalist_id_key(
        &self,
        untrusted: UntrustedJournalistIdPublicKey,
        now: DateTime<Utc>,
    ) -> Option<JournalistIdPublicKey> {
        self.journalist_provisioning_pk_iter()
            .find_map(|journalist_provisioning_pk| {
                untrusted.to_trusted(journalist_provisioning_pk, now).ok()
            })
    }

    pub fn verify_journalist_messaging_key(
        &self,
        untrusted: UntrustedJournalistMessagingPublicKey,
        now: DateTime<Utc>,
    ) -> Option<JournalistMessagingPublicKey> {
        self.journalist_id_pk_iter()
            .find_map(|(_, journalist_id_pk)| untrusted.to_trusted(journalist_id_pk, now).ok())
    }

    pub fn insert(&mut self, org_pk_family: OrganizationPublicKeyFamily) {
        self.0.push(org_pk_family);
    }
}
