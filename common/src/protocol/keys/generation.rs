use chrono::{DateTime, Duration, Utc};

use crate::{
    crypto::keys::{role::Role, signing::traits::PublicSigningKey},
    protocol::constants::{
        COVERNODE_ID_KEY_VALID_DURATION, COVERNODE_PROVISIONING_KEY_VALID_DURATION,
        JOURNALIST_ID_KEY_VALID_DURATION, JOURNALIST_PROVISIONING_KEY_VALID_DURATION,
        ORGANIZATION_KEY_VALID_DURATION,
    },
};

use crate::protocol::constants::{
    COVERNODE_MSG_KEY_VALID_DURATION, JOURNALIST_MSG_KEY_VALID_DURATION,
};

use super::*;

pub fn generate_child_expiry_not_valid_after<R: Role>(
    ttl_duration: Duration,
    parent_key_pair: &SignedSigningKeyPair<R>,
    now: DateTime<Utc>,
) -> DateTime<Utc> {
    let parent_not_valid_after = parent_key_pair.public_key().not_valid_after;
    let mut not_valid_after = now + ttl_duration;

    // A child key cannot outlive its parent otherwise there will not be a valid key to verify it
    if not_valid_after > parent_not_valid_after {
        tracing::warn!("New child key is expected to outlive its parent. This suggests that the parent key ({}) has not rotated quickly enough or that the wrong parent key is being used.", hex::encode(parent_key_pair.public_key().as_bytes()));

        not_valid_after = parent_not_valid_after;
    }

    not_valid_after
}

pub fn generate_organization_key_pair(now: DateTime<Utc>) -> OrganizationKeyPair {
    let not_valid_after = now + ORGANIZATION_KEY_VALID_DURATION;

    UnsignedSigningKeyPair::generate().to_self_signed_key_pair(not_valid_after)
}

/// Create a new signing key pair for the creation of new journalists
pub fn generate_journalist_provisioning_key_pair(
    org_key_pair: &OrganizationKeyPair,
    now: DateTime<Utc>,
) -> JournalistProvisioningKeyPair {
    let not_valid_after = generate_child_expiry_not_valid_after(
        JOURNALIST_PROVISIONING_KEY_VALID_DURATION,
        org_key_pair,
        now,
    );

    UnsignedSigningKeyPair::generate().to_signed_key_pair(org_key_pair, not_valid_after)
}

/// Create a new signing key pair for journalists to sign their encryption keys
pub fn generate_journalist_id_key_pair(
    journalist_provisioning_key_pair: &JournalistProvisioningKeyPair,
    now: DateTime<Utc>,
) -> JournalistIdKeyPair {
    let not_valid_after = generate_child_expiry_not_valid_after(
        JOURNALIST_ID_KEY_VALID_DURATION,
        journalist_provisioning_key_pair,
        now,
    );

    UnsignedSigningKeyPair::generate()
        .to_signed_key_pair(journalist_provisioning_key_pair, not_valid_after)
}

pub fn generate_unregistered_journalist_id_key_pair() -> UnregisteredJournalistIdKeyPair {
    UnsignedSigningKeyPair::generate()
}

/// Create a new encryption key pair with the public key's `not_valid_after` set to the
/// default period a journalist key is valid. This is equal to [`JOURNALIST_MSG_KEY_VALID_DURATION`] from `now()`
pub fn generate_journalist_messaging_key_pair(
    journalist_id_key_pair: &JournalistIdKeyPair,
    now: DateTime<Utc>,
) -> JournalistMessagingKeyPair {
    let not_valid_after = generate_child_expiry_not_valid_after(
        JOURNALIST_MSG_KEY_VALID_DURATION,
        journalist_id_key_pair,
        now,
    );

    UnsignedEncryptionKeyPair::generate()
        .to_signed_key_pair(journalist_id_key_pair, not_valid_after)
}

/// Create a new signing key pair for the creation of new CoverNodes
pub fn generate_covernode_provisioning_key_pair(
    org_key_pair: &OrganizationKeyPair,
    now: DateTime<Utc>,
) -> CoverNodeProvisioningKeyPair {
    let not_valid_after = generate_child_expiry_not_valid_after(
        COVERNODE_PROVISIONING_KEY_VALID_DURATION,
        org_key_pair,
        now,
    );

    UnsignedSigningKeyPair::generate().to_signed_key_pair(org_key_pair, not_valid_after)
}

/// Create a new signing key pair for the CoverNode to sign it's encryption keys
pub fn generate_covernode_id_key_pair(
    covernode_provisioning_key_pair: &CoverNodeProvisioningKeyPair,
    now: DateTime<Utc>,
) -> CoverNodeIdKeyPair {
    let not_valid_after = generate_child_expiry_not_valid_after(
        COVERNODE_ID_KEY_VALID_DURATION,
        covernode_provisioning_key_pair,
        now,
    );

    UnsignedSigningKeyPair::generate()
        .to_signed_key_pair(covernode_provisioning_key_pair, not_valid_after)
}

pub fn generate_unregistered_covernode_id_key_pair() -> UnregisteredCoverNodeIdKeyPair {
    tracing::info!("generating new unregistered id key pair");
    UnsignedSigningKeyPair::generate()
}

/// Create a new encryption key pair with the public key's `not_valid_after` set to the
/// default period a CoverNode key is valid. This is equal to [`COVERNODE_MSG_KEY_VALID_DURATION`] from `now()`
pub fn generate_covernode_messaging_key_pair(
    covernode_id_key_pair: &CoverNodeIdKeyPair,
    now: DateTime<Utc>,
) -> CoverNodeMessagingKeyPair {
    let not_valid_after = generate_child_expiry_not_valid_after(
        COVERNODE_MSG_KEY_VALID_DURATION,
        covernode_id_key_pair,
        now,
    );

    UnsignedEncryptionKeyPair::generate().to_signed_key_pair(covernode_id_key_pair, not_valid_after)
}

// Making this #[cfg(test)] seems to break our IDEs :(
pub mod test {
    use std::collections::HashMap;

    use chrono::{DateTime, Utc};

    use super::{
        generate_covernode_id_key_pair, generate_covernode_messaging_key_pair,
        generate_covernode_provisioning_key_pair, generate_journalist_id_key_pair,
        generate_journalist_messaging_key_pair, generate_journalist_provisioning_key_pair,
        generate_organization_key_pair,
    };
    use crate::backup::keys::{
        generate_backup_id_key_pair, generate_backup_msg_key_pair, BackupIdKeyPair,
        BackupMsgKeyPair,
    };
    use crate::protocol::keys::{BackupIdPublicKeyFamilyList, IdentityPublicKeyFamily};
    use crate::{
        api::models::{covernode_id::CoverNodeIdentity, journalist_id::JournalistIdentity},
        crypto::keys::encryption::UnsignedEncryptionKeyPair,
        protocol::{
            keys::{
                CoverDropPublicKeyHierarchy, CoverNodeIdKeyPair, CoverNodeIdPublicKey,
                CoverNodeIdPublicKeyFamily, CoverNodeIdPublicKeyFamilyList,
                CoverNodeMessagingKeyPair, CoverNodeMessagingPublicKey,
                CoverNodeProvisioningKeyPair, CoverNodeProvisioningPublicKey,
                CoverNodeProvisioningPublicKeyFamily, CoverNodeProvisioningPublicKeyFamilyList,
                JournalistIdKeyPair, JournalistIdPublicKey, JournalistIdPublicKeyFamily,
                JournalistIdPublicKeyFamilyList, JournalistMessagingKeyPair,
                JournalistMessagingPublicKey, JournalistProvisioningKeyPair,
                JournalistProvisioningPublicKey, JournalistProvisioningPublicKeyFamily,
                JournalistProvisioningPublicKeyFamilyList, OrganizationKeyPair,
                OrganizationPublicKey, OrganizationPublicKeyFamily, UserKeyPair, UserPublicKey,
            },
            roles::User,
        },
    };

    pub struct ProtocolKeys {
        pub org_pk: OrganizationPublicKey,
        pub org_key_pair: OrganizationKeyPair,
        pub user_pk: UserPublicKey,
        pub user_key_pair: UserKeyPair,
        pub covernode_provisioning_pk: CoverNodeProvisioningPublicKey,
        pub covernode_provisioning_key_pair: CoverNodeProvisioningKeyPair,
        pub covernode_id_pk: CoverNodeIdPublicKey,
        pub covernode_id_key_pair: CoverNodeIdKeyPair,
        pub covernode_msg_pk: CoverNodeMessagingPublicKey,
        pub covernode_msg_key_pair: CoverNodeMessagingKeyPair,
        pub journalist_provisioning_pk: JournalistProvisioningPublicKey,
        pub journalist_provisioning_key_pair: JournalistProvisioningKeyPair,
        pub journalist_id_pk: JournalistIdPublicKey,
        pub journalist_id_key_pair: JournalistIdKeyPair,
        pub journalist_msg_pk: JournalistMessagingPublicKey,
        pub journalist_msg_key_pair: JournalistMessagingKeyPair,
        pub backup_id_key_pair: BackupIdKeyPair,
        pub backup_msg_key_pair: BackupMsgKeyPair,
        pub hierarchy: CoverDropPublicKeyHierarchy,
    }

    impl ProtocolKeys {
        #[allow(clippy::too_many_arguments)]
        pub fn new(
            org_pk: OrganizationPublicKey,
            org_key_pair: OrganizationKeyPair,
            user_pk: UserPublicKey,
            user_key_pair: UserKeyPair,
            covernode_provisioning_pk: CoverNodeProvisioningPublicKey,
            covernode_provisioning_key_pair: CoverNodeProvisioningKeyPair,
            covernode_id_pk: CoverNodeIdPublicKey,
            covernode_id_key_pair: CoverNodeIdKeyPair,
            covernode_msg_pk: CoverNodeMessagingPublicKey,
            covernode_msg_key_pair: CoverNodeMessagingKeyPair,
            journalist_provisioning_pk: JournalistProvisioningPublicKey,
            journalist_provisioning_key_pair: JournalistProvisioningKeyPair,
            journalist_id_pk: JournalistIdPublicKey,
            journalist_id_key_pair: JournalistIdKeyPair,
            journalist_msg_pk: JournalistMessagingPublicKey,
            journalist_msg_key_pair: JournalistMessagingKeyPair,
            backup_id_key_pair: BackupIdKeyPair,
            backup_msg_key_pair: BackupMsgKeyPair,
            hierarchy: CoverDropPublicKeyHierarchy,
        ) -> Self {
            Self {
                org_pk,
                org_key_pair,
                user_pk,
                user_key_pair,
                covernode_provisioning_pk,
                covernode_provisioning_key_pair,
                covernode_id_pk,
                covernode_id_key_pair,
                covernode_msg_pk,
                covernode_msg_key_pair,
                journalist_provisioning_pk,
                journalist_provisioning_key_pair,
                journalist_id_pk,
                journalist_id_key_pair,
                journalist_msg_pk,
                journalist_msg_key_pair,
                backup_id_key_pair,
                backup_msg_key_pair,
                hierarchy,
            }
        }
    }

    // Helper function used in testing to create a clean fleet of keys
    pub fn generate_protocol_keys(now: DateTime<Utc>) -> ProtocolKeys {
        let org_key_pair = generate_organization_key_pair(now);

        // User
        let user_key_pair = UnsignedEncryptionKeyPair::<User>::generate();

        // CoverNode
        let covernode_provisioning_key_pair =
            generate_covernode_provisioning_key_pair(&org_key_pair, now);

        let covernode_id = CoverNodeIdentity::from_node_id(1);
        let covernode_id_key_pair =
            generate_covernode_id_key_pair(&covernode_provisioning_key_pair, now);
        let covernode_msg_key_pair =
            generate_covernode_messaging_key_pair(&covernode_id_key_pair, now);

        // Journalist
        let journalist_provisioning_key_pair =
            generate_journalist_provisioning_key_pair(&org_key_pair, now);
        let journalist_id_key_pair =
            generate_journalist_id_key_pair(&journalist_provisioning_key_pair, now);
        let journalist_msg_key_pair =
            generate_journalist_messaging_key_pair(&journalist_id_key_pair, now);

        // Backups
        let backup_id_key_pair = generate_backup_id_key_pair(&org_key_pair, now);
        let backup_msg_key_pair = generate_backup_msg_key_pair(&backup_id_key_pair, now);

        let hierarchy = CoverDropPublicKeyHierarchy::new(vec![OrganizationPublicKeyFamily::new(
            org_key_pair.public_key().clone(),
            CoverNodeProvisioningPublicKeyFamilyList::new(vec![
                CoverNodeProvisioningPublicKeyFamily::new(
                    covernode_provisioning_key_pair.public_key().clone(),
                    {
                        let mut map = HashMap::new();

                        let covernode_keys = CoverNodeIdPublicKeyFamilyList::new(vec![
                            CoverNodeIdPublicKeyFamily::new(
                                covernode_id_key_pair.public_key().clone(),
                                vec![covernode_msg_key_pair.public_key().clone()],
                            ),
                        ]);

                        map.insert(covernode_id, covernode_keys);

                        map
                    },
                ),
            ]),
            JournalistProvisioningPublicKeyFamilyList::new(vec![
                JournalistProvisioningPublicKeyFamily::new(
                    journalist_provisioning_key_pair.public_key().clone(),
                    HashMap::from([(
                        JournalistIdentity::new("journalist_0").unwrap(),
                        JournalistIdPublicKeyFamilyList::new(vec![
                            JournalistIdPublicKeyFamily::new(
                                journalist_id_key_pair.public_key().clone(),
                                vec![journalist_msg_key_pair.public_key().clone()],
                            ),
                        ]),
                    )]),
                ),
            ]),
            Some(BackupIdPublicKeyFamilyList::new(vec![
                IdentityPublicKeyFamily::new(
                    backup_id_key_pair.public_key().clone(),
                    vec![backup_msg_key_pair.public_key().clone()],
                ),
            ])),
        )]);

        ProtocolKeys::new(
            org_key_pair.public_key().clone(),
            org_key_pair,
            user_key_pair.public_key().clone(),
            user_key_pair,
            covernode_provisioning_key_pair.public_key().clone(),
            covernode_provisioning_key_pair,
            covernode_id_key_pair.public_key().clone(),
            covernode_id_key_pair,
            covernode_msg_key_pair.public_key().clone(),
            covernode_msg_key_pair,
            journalist_provisioning_key_pair.public_key().clone(),
            journalist_provisioning_key_pair,
            journalist_id_key_pair.public_key().clone(),
            journalist_id_key_pair,
            journalist_msg_key_pair.public_key().clone(),
            journalist_msg_key_pair,
            backup_id_key_pair,
            backup_msg_key_pair,
            hierarchy,
        )
    }

    #[test]
    // This is useful to run to see the output of key generation for debugging
    fn test_key_generation() {
        let now = Utc::now();
        let keys = generate_protocol_keys(now);
        print!("{:#?}", keys.hierarchy);
        assert!(keys.backup_id_key_pair.public_key().not_valid_after > now);
    }
}
