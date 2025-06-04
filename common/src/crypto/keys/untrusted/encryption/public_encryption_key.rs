use std::marker::PhantomData;

use hex_buffer_serde::Hex;
use serde::{Deserialize, Serialize};
use x25519_dalek::PublicKey as X25519PublicKey;

use crate::crypto::keys::{
    encryption::PublicEncryptionKey,
    public_key::PublicKey,
    role::Role,
    serde::{PublicEncryptionKeyHex, StorableKeyMaterial, StorableKeyMaterialType},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UntrustedPublicEncryptionKey<KeyRole: Role> {
    #[serde(with = "PublicEncryptionKeyHex")]
    pub key: X25519PublicKey,
    #[serde(skip)]
    marker: PhantomData<KeyRole>,
}

impl<KeyRole> UntrustedPublicEncryptionKey<KeyRole>
where
    KeyRole: Role,
{
    pub fn new(key: X25519PublicKey) -> Self {
        Self {
            key,
            marker: PhantomData,
        }
    }

    pub fn to_trusted(&self) -> PublicEncryptionKey<KeyRole> {
        PublicEncryptionKey::new(self.key)
    }
}

impl<KeyRole: Role> StorableKeyMaterial<'_, KeyRole> for UntrustedPublicEncryptionKey<KeyRole> {
    const TYPE: StorableKeyMaterialType = StorableKeyMaterialType::PublicKey;
}

impl<KeyRole: Role> PublicKey for UntrustedPublicEncryptionKey<KeyRole> {
    fn public_key_hex(&self) -> String {
        hex::encode(self.key.as_bytes())
    }
}
