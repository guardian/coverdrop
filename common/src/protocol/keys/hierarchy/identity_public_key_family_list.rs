use std::slice::Iter;

use chrono::{DateTime, Utc};

use crate::crypto::keys::{
    encryption::SignedPublicEncryptionKey,
    role::Role,
    signing::{traits, SignedPublicSigningKey},
    untrusted::{
        encryption::UntrustedSignedPublicEncryptionKey, signing::UntrustedSignedPublicSigningKey,
    },
};

use super::{IdentityPublicKeyFamily, UntrustedIdentityPublicKeyFamilyList};

/// A list of [`PublicKeyFamily`].
///
/// Verifying the keys should take `O(|num_id_keys| * |num_msg_key_per_id_key|).
/// The number of ID keys at any point in time should be no more than 2 under normal circumstances
/// so this isn't so bad.
///
/// [`PublicKeyFamily`]: super::PublicKeyFamily
#[derive(Clone, Debug)]
pub struct IdentityPublicKeyFamilyList<VerifyingRole: Role, IdentityRole: Role, MessagingRole: Role>(
    Vec<IdentityPublicKeyFamily<VerifyingRole, IdentityRole, MessagingRole>>,
);

impl<VerifyingRole: Role, IdentityRole: Role, MessagingRole: Role>
    IdentityPublicKeyFamilyList<VerifyingRole, IdentityRole, MessagingRole>
{
    pub fn empty() -> Self {
        Self(vec![])
    }

    pub fn new(
        keys: Vec<IdentityPublicKeyFamily<VerifyingRole, IdentityRole, MessagingRole>>,
    ) -> Self {
        Self(keys)
    }

    pub fn from_untrusted(
        keys: UntrustedIdentityPublicKeyFamilyList<VerifyingRole, IdentityRole, MessagingRole>,
        verifying_pk: &SignedPublicSigningKey<VerifyingRole>,
        now: DateTime<Utc>,
    ) -> Self {
        let keys = keys
            .into_iter()
            .flat_map(|untrusted_id_pk_family| {
                IdentityPublicKeyFamily::from_untrusted(untrusted_id_pk_family, verifying_pk, now)
            })
            .collect();

        Self(keys)
    }

    /// Consume a vector of ID keys and messaging keys, converting them into a list of key families.
    pub fn from_flat_key_vectors(
        verifying_pk: &impl traits::PublicSigningKey<VerifyingRole>,
        id_pks: Vec<UntrustedSignedPublicSigningKey<IdentityRole>>,
        msg_pks: Vec<UntrustedSignedPublicEncryptionKey<MessagingRole>>,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        let mut keys = vec![];

        for id_pk in id_pks {
            if let Ok(id_pk) = id_pk.to_trusted(verifying_pk, now) {
                let id_and_msg = IdentityPublicKeyFamily::<
                    VerifyingRole,
                    IdentityRole,
                    MessagingRole,
                >::new(id_pk, vec![]);
                keys.push(id_and_msg);
            } else {
                tracing::warn!("Unable to verify {} key", IdentityRole::display());
            }
        }

        'msg_pk_loop: for msg_pk in msg_pks.into_iter() {
            // Group the messaging key with the appropraite identity key
            for id_pk_with_msg_keys in &mut keys {
                if let Ok(verified_msg_pk) = msg_pk.to_trusted(&id_pk_with_msg_keys.id_pk, now) {
                    id_pk_with_msg_keys.msg_pks.push(verified_msg_pk);
                    continue 'msg_pk_loop;
                }
            }

            tracing::warn!(
                "No matched identity key for {} key",
                MessagingRole::display()
            )
        }

        Ok(Self(keys))
    }

    pub fn to_untrusted(
        &self,
    ) -> UntrustedIdentityPublicKeyFamilyList<VerifyingRole, IdentityRole, MessagingRole> {
        UntrustedIdentityPublicKeyFamilyList::new(
            self.iter()
                .map(|id_and_msg_keys| id_and_msg_keys.to_untrusted())
                .collect(),
        )
    }

    pub fn iter(
        &self,
    ) -> Iter<'_, IdentityPublicKeyFamily<VerifyingRole, IdentityRole, MessagingRole>> {
        self.0.iter()
    }

    pub fn id_pk_iter(&self) -> impl Iterator<Item = &SignedPublicSigningKey<IdentityRole>> + '_ {
        self.0.iter().map(|keys| &keys.id_pk)
    }

    pub fn msg_pk_iter(
        &self,
    ) -> impl Iterator<Item = &SignedPublicEncryptionKey<MessagingRole>> + '_ {
        self.0.iter().flat_map(|keys| keys.msg_pks.iter())
    }

    pub fn latest_msg_pk(&self) -> Option<&SignedPublicEncryptionKey<MessagingRole>> {
        self.0
            .iter()
            .flat_map(|keys| keys.msg_pks.iter())
            .max_by(|a, b| a.not_valid_after.cmp(&b.not_valid_after))
    }

    pub fn insert(
        &mut self,
        id_and_msg_keys: IdentityPublicKeyFamily<VerifyingRole, IdentityRole, MessagingRole>,
    ) {
        self.0.push(id_and_msg_keys)
    }
}
