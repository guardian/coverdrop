use chrono::{DateTime, Utc};
use ed25519_dalek::SigningKey;
use hex_buffer_serde::Hex;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::crypto::keys::{
    public_key::PublicKey,
    role::Role,
    serde::{SigningKeyPairHex, StorableKeyMaterial, StorableKeyMaterialType},
    signing::{traits, PublicSigningKey, SignedSigningKeyPair},
    untrusted::UntrustedKeyError,
};

use super::UntrustedSignedPublicSigningKey;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct UntrustedSignedSigningKeyPair<KeyRole: Role> {
    pub public_key: UntrustedSignedPublicSigningKey<KeyRole>,
    #[serde(with = "SigningKeyPairHex")]
    pub secret_key: SigningKey,
}

impl<KeyRole: Role> UntrustedSignedSigningKeyPair<KeyRole> {
    pub fn new(
        public_key: UntrustedSignedPublicSigningKey<KeyRole>,
        secret_key: SigningKey,
    ) -> Self {
        Self {
            public_key,
            secret_key,
        }
    }

    pub fn to_trusted<SigningKeyRole: Role>(
        &self,
        signing_pk: &impl traits::PublicSigningKey<SigningKeyRole>,
        now: DateTime<Utc>,
    ) -> anyhow::Result<SignedSigningKeyPair<KeyRole>> {
        let pk = self.public_key.to_trusted(signing_pk, now)?;

        Ok(SignedSigningKeyPair::new(pk, self.secret_key.clone()))
    }

    pub fn to_trusted_self_signed(
        &self,
        now: DateTime<Utc>,
    ) -> anyhow::Result<SignedSigningKeyPair<KeyRole>> {
        let public_key = PublicSigningKey::<KeyRole>::new(self.public_key.key);

        let pk = self.public_key.to_trusted(&public_key, now)?;

        Ok(SignedSigningKeyPair::new(pk, self.secret_key.clone()))
    }

    // For when we don't know the exact signing public key to use to verify an untrusted key, here
    // we provide an iterator of potential parent public keys to use to verify the untrusted key pair
    // and convert it to a trusted key pair
    pub fn to_trusted_from_candidate_parents<'a, SigningKeyRole: Role>(
        &self,
        mut signing_pk_iter: impl Iterator<
            Item = &'a (impl traits::PublicSigningKey<SigningKeyRole> + 'a),
        >,
        now: DateTime<Utc>,
    ) -> anyhow::Result<SignedSigningKeyPair<KeyRole>> {
        let Some(pk) =
            signing_pk_iter.find_map(|signing_pk| self.public_key.to_trusted(signing_pk, now).ok())
        else {
            anyhow::bail!(UntrustedKeyError::ParentKeyNotFound)
        };

        Ok(SignedSigningKeyPair::new(pk, self.secret_key.clone()))
    }
}

impl<KeyRole> StorableKeyMaterial<'_, KeyRole> for UntrustedSignedSigningKeyPair<KeyRole>
where
    KeyRole: Role + Serialize + DeserializeOwned,
{
    const TYPE: StorableKeyMaterialType = StorableKeyMaterialType::KeyPair;
}

impl<KeyRole: Role> PublicKey for UntrustedSignedSigningKeyPair<KeyRole> {
    fn public_key_hex(&self) -> String {
        hex::encode(self.public_key.key.as_bytes())
    }
}
