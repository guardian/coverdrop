use std::hash::Hash;
use std::marker::PhantomData;

use chrono::{DateTime, Utc};
use hex_buffer_serde::Hex;
use serde::{Deserialize, Serialize};

use crate::crypto::{
    keys::{
        key_certificate_data::KeyCertificateData,
        public_key::PublicKey,
        role::Role,
        serde::{PublicSigningKeyHex, SignatureHex, StorableKeyMaterial, StorableKeyMaterialType},
        signing::{traits, SignedPublicSigningKey},
        untrusted::UntrustedKeyError,
        Ed25519PublicKey,
    },
    Signature,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UntrustedSignedPublicSigningKey<KeyRole>
where
    KeyRole: Role,
{
    #[serde(with = "PublicSigningKeyHex")]
    pub key: Ed25519PublicKey,
    #[serde(with = "SignatureHex")]
    pub certificate: Signature<KeyCertificateData>,
    pub not_valid_after: DateTime<Utc>,
    #[serde(skip)]
    marker: PhantomData<KeyRole>,
}

impl<KeyRole> UntrustedSignedPublicSigningKey<KeyRole>
where
    KeyRole: Role,
{
    pub fn new(
        key: Ed25519PublicKey,
        certificate: Signature<KeyCertificateData>,
        not_valid_after: DateTime<Utc>,
    ) -> Self {
        Self {
            key,
            certificate,
            not_valid_after,
            marker: PhantomData,
        }
    }

    pub fn to_trusted<R: Role>(
        &self,
        signing_pk: &impl traits::PublicSigningKey<R>,
        now: DateTime<Utc>,
    ) -> anyhow::Result<SignedPublicSigningKey<KeyRole>> {
        if now > self.not_valid_after {
            Err(UntrustedKeyError::CertificateExpired.into())
        } else {
            let certificate_data =
                KeyCertificateData::new_for_signing_key(&self.key, self.not_valid_after);

            signing_pk.verify::<KeyCertificateData>(&certificate_data, &self.certificate, now)?;

            Ok(SignedPublicSigningKey::new(
                self.key,
                self.certificate.clone(),
                self.not_valid_after,
            ))
        }
    }
}

impl<KeyRole: Role> StorableKeyMaterial<'_, KeyRole> for UntrustedSignedPublicSigningKey<KeyRole> {
    const TYPE: StorableKeyMaterialType = StorableKeyMaterialType::PublicKey;
}

impl<KeyRole: Role> PublicKey for UntrustedSignedPublicSigningKey<KeyRole> {
    fn public_key_hex(&self) -> String {
        hex::encode(self.key.as_bytes())
    }
}

impl<KeyRole: Role> Hash for UntrustedSignedPublicSigningKey<KeyRole> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key.as_bytes().hash(state);
        self.certificate.hash(state);
        self.not_valid_after.hash(state);
    }
}
