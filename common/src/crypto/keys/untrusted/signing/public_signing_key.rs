use std::marker::PhantomData;

use hex_buffer_serde::Hex;
use serde::{Deserialize, Serialize};

use crate::crypto::keys::{
    public_key::PublicKey,
    role::Role,
    serde::{PublicSigningKeyHex, StorableKeyMaterial, StorableKeyMaterialType},
    signing::PublicSigningKey,
    Ed25519PublicKey,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UntrustedPublicSigningKey<KeyRole>
where
    KeyRole: Role,
{
    #[serde(with = "PublicSigningKeyHex")]
    pub key: Ed25519PublicKey,
    #[serde(skip)]
    marker: PhantomData<KeyRole>,
}

impl<KeyRole> UntrustedPublicSigningKey<KeyRole>
where
    KeyRole: Role,
{
    pub fn new(key: Ed25519PublicKey) -> Self {
        Self {
            key,
            marker: PhantomData,
        }
    }

    pub fn to_trusted(&self) -> PublicSigningKey<KeyRole> {
        PublicSigningKey::new(self.key)
    }
}

impl<KeyRole: Role> StorableKeyMaterial<'_, KeyRole> for UntrustedPublicSigningKey<KeyRole> {
    const TYPE: StorableKeyMaterialType = StorableKeyMaterialType::PublicKey;
}

impl<KeyRole: Role> PublicKey for UntrustedPublicSigningKey<KeyRole> {
    fn public_key_hex(&self) -> String {
        hex::encode(self.key.as_bytes())
    }
}
