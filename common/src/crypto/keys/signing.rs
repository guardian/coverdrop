use std::{
    hash::{Hash, Hasher},
    marker::PhantomData,
};

use chrono::{DateTime, Utc};

use ed25519_dalek::{Signer, SigningKey};
use rand::thread_rng;

use crate::crypto::{signable::Signable, signature::Signature};

use super::{
    key_certificate_data::KeyCertificateData,
    public_key::PublicKey,
    role::Role,
    untrusted::signing::{
        UntrustedPublicSigningKey, UntrustedSignedPublicSigningKey, UntrustedSignedSigningKeyPair,
        UntrustedUnsignedSigningKeyPair,
    },
    Ed25519PublicKey,
};

#[derive(Clone, Debug)]
pub struct PublicSigningKey<T: Role> {
    pub key: Ed25519PublicKey,
    marker: PhantomData<T>,
}

impl<T: Role> PublicSigningKey<T> {
    pub fn new(key: Ed25519PublicKey) -> Self {
        PublicSigningKey {
            key,
            marker: PhantomData,
        }
    }

    pub fn to_untrusted(&self) -> UntrustedPublicSigningKey<T> {
        UntrustedPublicSigningKey::new(self.key)
    }
}

impl<T: Role> Hash for PublicSigningKey<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key.as_bytes().hash(state);
    }
}

impl<T: Role> AsRef<Ed25519PublicKey> for PublicSigningKey<T> {
    fn as_ref(&self) -> &Ed25519PublicKey {
        &self.key
    }
}

impl<T: Role> PublicKey for PublicSigningKey<T> {
    fn public_key_hex(&self) -> String {
        hex::encode(self.key.as_bytes())
    }
}

#[derive(Clone, Debug, Eq)]
pub struct SignedPublicSigningKey<T: Role> {
    pub key: Ed25519PublicKey,
    pub certificate: Signature<KeyCertificateData>,
    pub not_valid_after: DateTime<Utc>,
    marker: PhantomData<T>,
}

impl<KeyRole: Role> PublicKey for SignedPublicSigningKey<KeyRole> {
    fn public_key_hex(&self) -> String {
        hex::encode(self.key.as_bytes())
    }
}

impl<T: Role> SignedPublicSigningKey<T> {
    pub fn new(
        key: Ed25519PublicKey,
        certificate: Signature<KeyCertificateData>,
        not_valid_after: DateTime<Utc>,
    ) -> Self {
        SignedPublicSigningKey {
            key,
            certificate,
            not_valid_after,
            marker: PhantomData::<T>,
        }
    }

    pub fn to_untrusted(&self) -> UntrustedSignedPublicSigningKey<T> {
        UntrustedSignedPublicSigningKey::new(
            self.key,
            self.certificate.clone(),
            self.not_valid_after,
        )
    }
}

// Allow comparing public signing keys with different roles.
// This is used to compare the `TrustedOrganizationPublicKey` with an `OrganizationPublicKey`
impl<R1: Role, R2: Role> PartialEq<SignedPublicSigningKey<R2>> for SignedPublicSigningKey<R1> {
    fn eq(&self, other: &SignedPublicSigningKey<R2>) -> bool {
        self.key == other.key
            && self.certificate == other.certificate
            && self.not_valid_after == other.not_valid_after
    }
}

impl<T: Role> AsRef<Ed25519PublicKey> for SignedPublicSigningKey<T> {
    fn as_ref(&self) -> &Ed25519PublicKey {
        &self.key
    }
}

impl<T: Role> Hash for SignedPublicSigningKey<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key.as_bytes().hash(state);
        self.certificate.to_bytes().hash(state);
    }
}

pub type SignedSigningKeyPair<R> = SigningKeyPair<R, SignedPublicSigningKey<R>>;
pub type UnsignedSigningKeyPair<R> = SigningKeyPair<R, PublicSigningKey<R>>;

// Specialised functions for the UNSIGNED version of a key pair
impl<R: Role> UnsignedSigningKeyPair<R> {
    pub fn to_signed_key_pair<SR, SK>(
        self,
        signing_key_pair: &SigningKeyPair<SR, SK>,
        not_valid_after: DateTime<Utc>,
    ) -> SignedSigningKeyPair<R>
    where
        SR: Role,
        SK: traits::PublicSigningKey<SR>,
    {
        let cert_data =
            KeyCertificateData::new_for_signing_key(&self.public_key.key, not_valid_after);
        let certificate = signing_key_pair.sign(&cert_data);

        let pk = SignedPublicSigningKey::new(self.public_key.key, certificate, not_valid_after);

        SignedSigningKeyPair::<R>::new(pk, self.secret_key)
    }

    pub fn to_self_signed_key_pair(
        self,
        not_valid_after: DateTime<Utc>,
    ) -> SignedSigningKeyPair<R> {
        let cert_data =
            KeyCertificateData::new_for_signing_key(&self.public_key.key, not_valid_after);
        let certificate = self.sign(&cert_data);

        let pk = SignedPublicSigningKey::new(self.public_key.key, certificate, not_valid_after);

        SignedSigningKeyPair::<R>::new(pk, self.secret_key)
    }

    pub fn to_untrusted(&self) -> UntrustedUnsignedSigningKeyPair<R> {
        UntrustedUnsignedSigningKeyPair::new(
            self.public_key.to_untrusted(),
            self.secret_key.clone(),
        )
    }
}

impl<R: Role> SignedSigningKeyPair<R> {
    pub fn to_untrusted(&self) -> UntrustedSignedSigningKeyPair<R> {
        UntrustedSignedSigningKeyPair::new(self.public_key.to_untrusted(), self.secret_key.clone())
    }
}

#[derive(Clone)]
pub struct SigningKeyPair<R, K>
where
    K: traits::PublicSigningKey<R>,
    R: Role,
{
    public_key: K,
    pub secret_key: SigningKey,
    marker: PhantomData<R>,
}

impl<R, K> SigningKeyPair<R, K>
where
    R: Role,
    K: traits::PublicSigningKey<R>,
{
    /// Useful for converting from one role type of key pair to another.
    pub fn new(pk: K, secret_key: SigningKey) -> Self {
        Self {
            public_key: pk,
            secret_key,
            marker: PhantomData,
        }
    }

    /// Generate a random key pair to be used for signing.
    pub fn generate() -> SigningKeyPair<R, PublicSigningKey<R>> {
        let mut csprng = thread_rng();
        let key_pair = SigningKey::generate(&mut csprng);

        let pk = PublicSigningKey::<R>::new(key_pair.verifying_key());

        SigningKeyPair::<R, PublicSigningKey<R>>::new(pk, key_pair)
    }

    /// Sign some [`Signable`] message the resultant signature will track the type of the signable
    /// to provide type safety to the signatures.
    ///
    /// [`Signable`]: crate::crypto::Signable
    pub fn sign<S>(&self, signable: &S) -> Signature<S>
    where
        S: Signable,
    {
        let signature = self.secret_key.sign(signable.as_signable_bytes());

        Signature {
            signature,
            marker: PhantomData,
        }
    }

    pub fn public_key(&self) -> &K {
        &self.public_key
    }

    pub fn to_public_key(&self) -> K {
        self.public_key.clone()
    }
}

impl<KeyRole, K> PublicKey for SigningKeyPair<KeyRole, K>
where
    KeyRole: Role,
    K: traits::PublicSigningKey<KeyRole>,
{
    fn public_key_hex(&self) -> String {
        hex::encode(self.public_key.raw_public_key().as_bytes())
    }
}

pub mod traits {
    use chrono::{DateTime, Utc};
    use ed25519_dalek::Verifier;

    use crate::{
        crypto::{
            keys::{role::Role, Ed25519PublicKey},
            Signable, Signature,
        },
        protocol::constants::ED25519_PUBLIC_KEY_LEN,
        Error,
    };

    use super::SigningKeyPair;

    pub trait PublicSigningKey<T: Role>: Clone {
        fn verify<S>(
            &self,
            message: &S,
            signature: &Signature<S>,
            now: DateTime<Utc>,
        ) -> Result<(), Error>
        where
            S: Signable;

        fn raw_public_key(&self) -> Ed25519PublicKey;

        fn as_bytes(&self) -> &[u8; ED25519_PUBLIC_KEY_LEN];
    }

    impl<T: Role> PublicSigningKey<T> for super::PublicSigningKey<T> {
        fn verify<S>(
            &self,
            message: &S,
            signature: &Signature<S>,
            _now: DateTime<Utc>, // The time is ignored for an unsigned key
        ) -> Result<(), Error>
        where
            S: Signable,
        {
            Ok(self
                .key
                .verify(message.as_signable_bytes(), &signature.signature)?)
        }

        fn raw_public_key(&self) -> Ed25519PublicKey {
            self.key
        }

        fn as_bytes(&self) -> &[u8; ED25519_PUBLIC_KEY_LEN] {
            self.key.as_bytes()
        }
    }

    impl<T: Role> PublicSigningKey<T> for super::SignedPublicSigningKey<T> {
        fn verify<S>(
            &self,
            message: &S,
            signature: &Signature<S>,
            now: DateTime<Utc>,
        ) -> Result<(), Error>
        where
            S: Signable,
        {
            if self.not_valid_after >= now {
                Ok(self
                    .key
                    .verify(message.as_signable_bytes(), &signature.signature)?)
            } else {
                tracing::debug!(
                    "Attempted to use expired signing key, expired at {}",
                    self.not_valid_after
                );
                Err(Error::SigningKeyExpired)
            }
        }

        fn raw_public_key(&self) -> Ed25519PublicKey {
            self.key
        }

        fn as_bytes(&self) -> &[u8; ED25519_PUBLIC_KEY_LEN] {
            self.key.as_bytes()
        }
    }

    impl<T: Role, K: PublicSigningKey<T>> PublicSigningKey<T> for SigningKeyPair<T, K> {
        fn verify<S>(
            &self,
            message: &S,
            signature: &Signature<S>,
            now: DateTime<Utc>,
        ) -> Result<(), Error>
        where
            S: Signable,
        {
            self.public_key().verify(message, signature, now)
        }

        fn raw_public_key(&self) -> Ed25519PublicKey {
            self.public_key.raw_public_key()
        }

        fn as_bytes(&self) -> &[u8; ED25519_PUBLIC_KEY_LEN] {
            self.public_key.as_bytes()
        }
    }
}
