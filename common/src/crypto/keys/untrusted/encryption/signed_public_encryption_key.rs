use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

use chrono::{DateTime, Utc};
use hex_buffer_serde::Hex;
use serde::{Deserialize, Serialize};
use x25519_dalek::PublicKey as X25519PublicKey;

use crate::crypto::keys::public_key::PublicKey;
use crate::crypto::keys::serde::StorableKeyMaterialType;
use crate::crypto::{
    keys::{
        encryption::{PublicEncryptionKey, SignedPublicEncryptionKey},
        key_certificate_data::KeyCertificateData,
        role::Role,
        serde::{PublicEncryptionKeyHex, SignatureHex, StorableKeyMaterial},
        signing::traits,
        untrusted::UntrustedKeyError,
    },
    Signature,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UntrustedSignedPublicEncryptionKey<KeyRole: Role> {
    #[serde(with = "PublicEncryptionKeyHex")]
    pub key: X25519PublicKey,
    #[serde(with = "SignatureHex")]
    pub certificate: Signature<KeyCertificateData>,
    pub not_valid_after: DateTime<Utc>,
    #[serde(skip)]
    marker: PhantomData<KeyRole>,
}

impl<KeyRole> UntrustedSignedPublicEncryptionKey<KeyRole>
where
    KeyRole: Role,
{
    pub fn new(
        key: X25519PublicKey,
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

    pub(crate) fn to_trusted<VerifyingRole: Role>(
        &self,
        signer_pk: &impl traits::PublicSigningKey<VerifyingRole>,
        now: DateTime<Utc>,
    ) -> Result<SignedPublicEncryptionKey<KeyRole>, UntrustedKeyError> {
        if now > self.not_valid_after {
            Err(UntrustedKeyError::CertificateExpired)
        } else {
            let certificate_data =
                KeyCertificateData::new_for_encryption_key(&self.key, self.not_valid_after);

            if signer_pk
                .verify::<KeyCertificateData>(&certificate_data, &self.certificate, now)
                .is_ok()
            {
                Ok(SignedPublicEncryptionKey::new(
                    PublicEncryptionKey::new(self.key),
                    self.certificate.clone(),
                    self.not_valid_after,
                ))
            } else {
                Err(UntrustedKeyError::CertificateNotValid)
            }
        }
    }
}

impl<KeyRole: Role> StorableKeyMaterial<'_, KeyRole>
    for UntrustedSignedPublicEncryptionKey<KeyRole>
{
    const TYPE: StorableKeyMaterialType = StorableKeyMaterialType::PublicKey;
}

impl<KeyRole: Role> PublicKey for UntrustedSignedPublicEncryptionKey<KeyRole> {
    fn public_key_hex(&self) -> String {
        hex::encode(self.key.as_bytes())
    }
}

impl<KeyRole: Role> Hash for UntrustedSignedPublicEncryptionKey<KeyRole> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key.as_bytes().hash(state);
        self.certificate.hash(state);
        self.not_valid_after.hash(state);
    }
}
