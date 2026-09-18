use std::hash::Hash;
use std::marker::PhantomData;

use chrono::{DateTime, Utc};
use hex_buffer_serde::Hex;
use serde::{Deserialize, Serialize};

use crate::{
    api::models::identity::Identity,
    crypto::{
        keys::{
            id_key_certificate_data::IdKeyCertificateData,
            key_certificate_data::KeyCertificateData,
            public_key::PublicKey,
            role::Role,
            serde::{
                OptionalSignatureHex, PublicSigningKeyHex, SignatureHex, StorableKeyMaterial,
                StorableKeyMaterialType,
            },
            signing::{traits, SignedPublicSigningKey},
            untrusted::UntrustedKeyError,
            Ed25519PublicKey,
        },
        Signature,
    },
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UntrustedSignedPublicSigningKey<KeyRole>
where
    KeyRole: Role,
{
    #[serde(with = "PublicSigningKeyHex")]
    pub key: Ed25519PublicKey,
    // deprecating `certificate` in favor of `signature` field
    // TODO (https://github.com/guardian/coverdrop-internal/issues/4200) remove once all keys have been migrated to include the signature field
    #[serde(with = "SignatureHex")]
    pub certificate: Signature<KeyCertificateData>,
    pub not_valid_after: DateTime<Utc>,
    // signature is Option for backwards compatibility.
    // TODO (https://github.com/guardian/coverdrop-internal/issues/4200) remove Option once all keys have been migrated to include the signature field
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "OptionalSignatureHex"
    )]
    pub signature: Option<Signature<KeyRole::CertData>>,
    #[serde(skip)]
    marker: PhantomData<KeyRole>,
}

impl<KeyRole: Role> UntrustedSignedPublicSigningKey<KeyRole> {
    pub fn new(
        key: Ed25519PublicKey,
        signature: Option<Signature<KeyRole::CertData>>,
        certificate: Signature<KeyCertificateData>,
        not_valid_after: DateTime<Utc>,
    ) -> Self {
        Self {
            key,
            certificate,
            not_valid_after,
            signature,
            marker: PhantomData,
        }
    }
}

impl<KeyRole: Role<CertData = KeyCertificateData>> UntrustedSignedPublicSigningKey<KeyRole> {
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

            // Verify new signature if it is present
            if let Some(signature) = &self.signature {
                signing_pk.verify::<KeyRole::CertData>(&certificate_data, signature, now)?;
            }
            // Also verify the legacy certificate until its removed
            // TODO (https://github.com/guardian/coverdrop-internal/issues/4200) remove legacy certificate verification
            signing_pk.verify::<KeyRole::CertData>(&certificate_data, &self.certificate, now)?;

            Ok(SignedPublicSigningKey::new(
                self.key,
                self.signature.clone(),
                self.certificate.clone(),
                self.not_valid_after,
            ))
        }
    }
}

impl<KeyRole: Role<CertData = IdKeyCertificateData>> UntrustedSignedPublicSigningKey<KeyRole> {
    pub fn to_trusted_with_identity<R: Role>(
        &self,
        signing_pk: &impl traits::PublicSigningKey<R>,
        now: DateTime<Utc>,
        identity: &impl Identity,
    ) -> anyhow::Result<SignedPublicSigningKey<KeyRole>> {
        if now > self.not_valid_after {
            Err(UntrustedKeyError::CertificateExpired.into())
        } else {
            // Verify new signature with identity if it is present
            if let Some(ref signature) = self.signature {
                let new_cert_data =
                    IdKeyCertificateData::new::<KeyRole>(&self.key, self.not_valid_after, identity);
                signing_pk.verify(&new_cert_data, signature, now)?;
            }
            // Also verify the legacy certificate until its removed
            // TODO (https://github.com/guardian/coverdrop-internal/issues/4200) remove legacy certificate verification
            let legacy_certificate_data =
                KeyCertificateData::new_for_signing_key(&self.key, self.not_valid_after);
            signing_pk.verify::<KeyCertificateData>(
                &legacy_certificate_data,
                &self.certificate,
                now,
            )?;

            Ok(SignedPublicSigningKey::new(
                self.key,
                self.signature.clone(),
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
