use chrono::{DateTime, Utc};
use hex_buffer_serde::Hex;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::crypto::keys::{
    encryption::{SecretEncryptionKey, SignedEncryptionKeyPair},
    public_key::PublicKey,
    role::Role,
    serde::{SecretEncryptionKeyHex, StorableKeyMaterial, StorableKeyMaterialType},
    signing,
    untrusted::UntrustedKeyError,
    X25519SecretKey,
};

use super::UntrustedSignedPublicEncryptionKey;

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UntrustedSignedEncryptionKeyPair<R: Role> {
    pub public_key: UntrustedSignedPublicEncryptionKey<R>,
    #[serde(with = "SecretEncryptionKeyHex")]
    pub secret_key: X25519SecretKey,
}

impl<KeyRole: Role> UntrustedSignedEncryptionKeyPair<KeyRole> {
    pub fn new(
        public_key: UntrustedSignedPublicEncryptionKey<KeyRole>,
        secret_key: X25519SecretKey,
    ) -> Self {
        Self {
            public_key,
            secret_key,
        }
    }

    pub fn to_trusted<SigningKeyRole: Role>(
        &self,
        signing_pk: &impl signing::traits::PublicSigningKey<SigningKeyRole>,
        now: DateTime<Utc>,
    ) -> anyhow::Result<SignedEncryptionKeyPair<KeyRole>> {
        let pk = self.public_key.to_trusted(signing_pk, now)?;

        let sk = SecretEncryptionKey::new(self.secret_key.clone());

        Ok(SignedEncryptionKeyPair::new(pk, sk))
    }
    // For when we don't know the exact signing public key to use to verify an untrusted key, here
    // we provide an iterator of potential parent public keys to use to verify the untrusted key pair
    // and convert it to a trusted key pair
    pub fn to_trusted_from_candidate_parents<'a, SigningKeyRole: Role>(
        &self,
        mut signing_pk_iter: impl Iterator<
            Item = &'a (impl signing::traits::PublicSigningKey<SigningKeyRole> + 'a),
        >,
        now: DateTime<Utc>,
    ) -> anyhow::Result<SignedEncryptionKeyPair<KeyRole>> {
        let Some(pk) =
            signing_pk_iter.find_map(|signing_pk| self.public_key.to_trusted(signing_pk, now).ok())
        else {
            anyhow::bail!(UntrustedKeyError::ParentKeyNotFound)
        };

        let sk = SecretEncryptionKey::new(self.secret_key.clone());

        Ok(SignedEncryptionKeyPair::new(pk, sk))
    }
}

impl<KeyRole> StorableKeyMaterial<'_, KeyRole> for UntrustedSignedEncryptionKeyPair<KeyRole>
where
    KeyRole: Role + Serialize + DeserializeOwned,
{
    const TYPE: StorableKeyMaterialType = StorableKeyMaterialType::KeyPair;
}

impl<KeyRole: Role> PublicKey for UntrustedSignedEncryptionKeyPair<KeyRole> {
    fn public_key_hex(&self) -> String {
        hex::encode(self.public_key.key.as_bytes())
    }
}
