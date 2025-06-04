use ed25519_dalek::SigningKey;
use hex_buffer_serde::Hex;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::crypto::keys::{
    public_key::PublicKey,
    role::Role,
    serde::{SigningKeyPairHex, StorableKeyMaterial, StorableKeyMaterialType},
    signing::UnsignedSigningKeyPair,
};

use super::UntrustedPublicSigningKey;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UntrustedUnsignedSigningKeyPair<R: Role> {
    pub public_key: UntrustedPublicSigningKey<R>,
    #[serde(with = "SigningKeyPairHex")]
    pub secret_key: SigningKey,
}

impl<KeyRole: Role> UntrustedUnsignedSigningKeyPair<KeyRole> {
    pub fn new(public_key: UntrustedPublicSigningKey<KeyRole>, secret_key: SigningKey) -> Self {
        Self {
            public_key,
            secret_key,
        }
    }

    pub fn to_trusted(&self) -> UnsignedSigningKeyPair<KeyRole> {
        let pk = self.public_key.to_trusted();

        UnsignedSigningKeyPair::new(pk, self.secret_key.clone())
    }
}

impl<KeyRole> StorableKeyMaterial<'_, KeyRole> for UntrustedUnsignedSigningKeyPair<KeyRole>
where
    KeyRole: Role + Serialize + DeserializeOwned,
{
    const TYPE: StorableKeyMaterialType = StorableKeyMaterialType::KeyPair;
}

impl<KeyRole: Role> PublicKey for UntrustedUnsignedSigningKeyPair<KeyRole> {
    fn public_key_hex(&self) -> String {
        hex::encode(self.public_key.key.as_bytes())
    }
}
