use hex_buffer_serde::Hex;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::crypto::keys::{
    encryption::{SecretEncryptionKey, UnsignedEncryptionKeyPair},
    public_key::PublicKey,
    role::Role,
    serde::{SecretEncryptionKeyHex, StorableKeyMaterial, StorableKeyMaterialType},
    X25519SecretKey,
};

use super::UntrustedPublicEncryptionKey;

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UntrustedUnsignedEncryptionKeyPair<R: Role> {
    pub public_key: UntrustedPublicEncryptionKey<R>,
    // When serialized we only store the secret key - so rename
    #[serde(with = "SecretEncryptionKeyHex")]
    pub secret_key: X25519SecretKey,
}

impl<KeyRole: Role> UntrustedUnsignedEncryptionKeyPair<KeyRole> {
    pub fn new(
        public_key: UntrustedPublicEncryptionKey<KeyRole>,
        secret_key: X25519SecretKey,
    ) -> Self {
        Self {
            public_key,
            secret_key,
        }
    }

    pub fn to_trusted(&self) -> UnsignedEncryptionKeyPair<KeyRole> {
        let pk = self.public_key.to_trusted();

        let sk = SecretEncryptionKey::new(self.secret_key.clone());

        UnsignedEncryptionKeyPair::new(pk, sk)
    }
}

impl<KeyRole> StorableKeyMaterial<'_, KeyRole> for UntrustedUnsignedEncryptionKeyPair<KeyRole>
where
    KeyRole: Role + Serialize + DeserializeOwned,
{
    const TYPE: StorableKeyMaterialType = StorableKeyMaterialType::KeyPair;
}

impl<KeyRole: Role> PublicKey for UntrustedUnsignedEncryptionKeyPair<KeyRole> {
    fn public_key_hex(&self) -> String {
        hex::encode(self.public_key.key.as_bytes())
    }
}
