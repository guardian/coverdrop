use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use crate::crypto::keys::{
    role::Role,
    untrusted::{
        encryption::UntrustedSignedPublicEncryptionKey, signing::UntrustedSignedPublicSigningKey,
    },
};

/// The "published" (aka stored on-disk or sent over a network) representation of the [`PublicKeyFamily`].
/// See the those docs for more details.
///
/// Must be verified before use.
///
/// [`PublicKeyFamily`]: super::PublicKeyFamily
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UntrustedIdentityPublicKeyFamily<
    VerifyingRole: Role,
    IdentityRole: Role,
    MessagingRole: Role,
> {
    pub id_pk: UntrustedSignedPublicSigningKey<IdentityRole>,
    pub msg_pks: Vec<UntrustedSignedPublicEncryptionKey<MessagingRole>>,
    #[serde(skip)]
    verifying_role_marker: PhantomData<VerifyingRole>,
}

impl<VerifyingRole: Role, IdentityRole: Role, MessagingRole: Role>
    UntrustedIdentityPublicKeyFamily<VerifyingRole, IdentityRole, MessagingRole>
{
    pub fn new(
        id_pk: UntrustedSignedPublicSigningKey<IdentityRole>,
        msg_pks: Vec<UntrustedSignedPublicEncryptionKey<MessagingRole>>,
    ) -> Self {
        Self {
            id_pk,
            msg_pks,
            verifying_role_marker: PhantomData,
        }
    }
}
