use std::slice::Iter;

use serde::{Deserialize, Serialize};

use crate::crypto::keys::{
    role::Role,
    untrusted::{
        encryption::UntrustedSignedPublicEncryptionKey, signing::UntrustedSignedPublicSigningKey,
    },
};

use super::UntrustedIdentityPublicKeyFamily;

/// A list of [`StoredPublicKeyFamily`].
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent, deny_unknown_fields)]
pub struct UntrustedIdentityPublicKeyFamilyList<
    VerifyingRole: Role,
    IdentityRole: Role,
    MessagingRole: Role,
>(Vec<UntrustedIdentityPublicKeyFamily<VerifyingRole, IdentityRole, MessagingRole>>);

impl<VerifyingRole: Role, IdentityRole: Role, MessagingRole: Role>
    UntrustedIdentityPublicKeyFamilyList<VerifyingRole, IdentityRole, MessagingRole>
{
    pub fn new(
        keys: Vec<UntrustedIdentityPublicKeyFamily<VerifyingRole, IdentityRole, MessagingRole>>,
    ) -> Self {
        Self(keys)
    }

    pub fn iter(
        &self,
    ) -> Iter<'_, UntrustedIdentityPublicKeyFamily<VerifyingRole, IdentityRole, MessagingRole>>
    {
        self.0.iter()
    }

    pub fn id_pk_iter(
        &self,
    ) -> impl Iterator<Item = &UntrustedSignedPublicSigningKey<IdentityRole>> + '_ {
        self.0.iter().map(|keys| &keys.id_pk)
    }

    pub fn msg_pk_iter(
        &self,
    ) -> impl Iterator<Item = &UntrustedSignedPublicEncryptionKey<MessagingRole>> {
        self.0.iter().flat_map(|k| k.msg_pks.iter())
    }
}

impl<VerifyingRole: Role, IdentityRole: Role, MessagingRole: Role> IntoIterator
    for UntrustedIdentityPublicKeyFamilyList<VerifyingRole, IdentityRole, MessagingRole>
{
    type Item = UntrustedIdentityPublicKeyFamily<VerifyingRole, IdentityRole, MessagingRole>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
