use std::marker::PhantomData;

use chrono::{DateTime, Utc};

use crate::crypto::keys::{
    encryption::SignedPublicEncryptionKey,
    role::Role,
    signing::{traits, SignedPublicSigningKey},
};

use super::UntrustedIdentityPublicKeyFamily;

/// Due to keys being refreshed we will occasionally have a situation where a journalist or piece of infrastructure
/// (e.g. the CoverNode) has multiple identity keys, which have multiple messaging keys. This structure deals with
/// grouping messaging keys to their respective identity key. The identity key is a signed key, usually signed by
/// the `Organization` key. The messaging keys are signed by the identity key. This allows us to have a chain of trust
/// up to the organization level.
///
/// The Family takes two `Role`s, `IR` which is the ID key `Role` and `MR` the messaging key's `Role`.
///
/// This version of the structure is for verified keys.
#[derive(Clone, Debug)]
pub struct IdentityPublicKeyFamily<VerifyingRole: Role, IdentityRole: Role, MessagingRole: Role> {
    pub id_pk: SignedPublicSigningKey<IdentityRole>,
    pub msg_pks: Vec<SignedPublicEncryptionKey<MessagingRole>>,
    verifying_role_marker: PhantomData<VerifyingRole>,
}

impl<VerifyingRole: Role, IdentityRole: Role, MessagingRole: Role>
    IdentityPublicKeyFamily<VerifyingRole, IdentityRole, MessagingRole>
{
    pub fn new(
        id_pk: SignedPublicSigningKey<IdentityRole>,
        msg_pks: Vec<SignedPublicEncryptionKey<MessagingRole>>,
    ) -> Self {
        Self {
            id_pk,
            msg_pks,
            verifying_role_marker: PhantomData,
        }
    }

    pub fn from_untrusted(
        untrusted: UntrustedIdentityPublicKeyFamily<VerifyingRole, IdentityRole, MessagingRole>,
        id_verifying_key: &impl traits::PublicSigningKey<VerifyingRole>,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        let id_pk = untrusted.id_pk.to_trusted(id_verifying_key, now)?;

        let msg_pks = untrusted
            .msg_pks
            .into_iter()
            .flat_map(|msg_pk| msg_pk.to_trusted(&id_pk, now))
            .collect();

        Ok(Self {
            id_pk,
            msg_pks,
            verifying_role_marker: PhantomData,
        })
    }

    pub fn to_untrusted(
        &self,
    ) -> UntrustedIdentityPublicKeyFamily<VerifyingRole, IdentityRole, MessagingRole> {
        UntrustedIdentityPublicKeyFamily::new(
            self.id_pk.to_untrusted(),
            self.msg_pks
                .iter()
                .map(|msg_pk| msg_pk.to_untrusted())
                .collect(),
        )
    }
}
