use std::marker::PhantomData;

use chrono::prelude::*;
use hex_buffer_serde::Hex;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret as X25519SecretKey};

use super::{
    key_certificate_data::KeyCertificateData,
    public_key::PublicKey,
    role::Role,
    serde::SecretEncryptionKeyHex,
    signing::{self, SigningKeyPair},
    untrusted::encryption::{
        UntrustedPublicEncryptionKey, UntrustedSignedEncryptionKeyPair,
        UntrustedSignedPublicEncryptionKey, UntrustedUnsignedEncryptionKeyPair,
    },
};
use crate::{
    crypto::signature::Signature,
    protocol::constants::{X25519_PUBLIC_KEY_LEN, X25519_SECRET_KEY_LEN},
    Error,
};

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct PublicEncryptionKey<T: Role> {
    pub key: X25519PublicKey,
    marker: PhantomData<T>,
}

impl<T: Role> PublicEncryptionKey<T> {
    pub fn new(key: X25519PublicKey) -> Self {
        PublicEncryptionKey {
            key,
            marker: PhantomData,
        }
    }

    pub fn from_fixed_bytes(bytes: [u8; 32]) -> Self {
        PublicEncryptionKey {
            key: X25519PublicKey::from(bytes),
            marker: PhantomData,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let byte_array: [u8; 32] = bytes.try_into().map_err(|_| Error::InvalidPublicKeyHex)?;

        Ok(PublicEncryptionKey {
            key: X25519PublicKey::from(byte_array),
            marker: PhantomData,
        })
    }

    pub fn to_untrusted(&self) -> UntrustedPublicEncryptionKey<T> {
        UntrustedPublicEncryptionKey::new(self.key)
    }
}

impl<T: Role> AsRef<X25519PublicKey> for PublicEncryptionKey<T> {
    fn as_ref(&self) -> &X25519PublicKey {
        &self.key
    }
}

impl<T: Role> From<&PublicEncryptionKey<T>> for PublicEncryptionKey<T> {
    fn from(value: &PublicEncryptionKey<T>) -> Self {
        value.clone()
    }
}

impl<R> PublicKey for PublicEncryptionKey<R>
where
    R: Role,
{
    fn public_key_hex(&self) -> String {
        hex::encode(self.key.as_bytes())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SignedPublicEncryptionKey<T: Role> {
    pub key: X25519PublicKey,
    pub certificate: Signature<KeyCertificateData>,
    pub not_valid_after: DateTime<Utc>,
    marker: PhantomData<T>,
}

impl<T: Role> SignedPublicEncryptionKey<T> {
    pub fn new(
        key: PublicEncryptionKey<T>,
        certificate: Signature<KeyCertificateData>,
        not_valid_after: DateTime<Utc>,
    ) -> Self {
        SignedPublicEncryptionKey {
            key: key.key,
            certificate,
            not_valid_after,
            marker: PhantomData,
        }
    }

    pub fn to_untrusted(&self) -> UntrustedSignedPublicEncryptionKey<T> {
        UntrustedSignedPublicEncryptionKey::new(
            self.key,
            self.certificate.clone(),
            self.not_valid_after,
        )
    }

    pub fn to_public_encryption_key(self) -> PublicEncryptionKey<T> {
        PublicEncryptionKey {
            key: self.key,
            marker: PhantomData,
        }
    }
}

impl<T: Role> AsRef<X25519PublicKey> for SignedPublicEncryptionKey<T> {
    fn as_ref(&self) -> &X25519PublicKey {
        &self.key
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SecretEncryptionKey<T: Role> {
    #[serde(with = "SecretEncryptionKeyHex")]
    pub key: X25519SecretKey,
    #[serde(skip)]
    marker: PhantomData<T>,
}

impl<T: Role> SecretEncryptionKey<T> {
    pub fn new(key: X25519SecretKey) -> Self {
        SecretEncryptionKey {
            key,
            marker: PhantomData,
        }
    }

    pub fn to_bytes(&self) -> [u8; X25519_SECRET_KEY_LEN] {
        self.key.to_bytes()
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let byte_array: [u8; 32] = bytes.try_into().map_err(|_| Error::InvalidPublicKeyHex)?;

        Ok(Self {
            key: X25519SecretKey::from(byte_array),
            marker: PhantomData,
        })
    }

    pub fn to_public_key(&self) -> PublicEncryptionKey<T> {
        let pk = X25519PublicKey::from(&self.key);
        PublicEncryptionKey::new(pk)
    }
}

impl<T: Role> AsRef<X25519SecretKey> for SecretEncryptionKey<T> {
    fn as_ref(&self) -> &X25519SecretKey {
        &self.key
    }
}

#[derive(Clone)]
pub struct EncryptionKeyPair<R, K>
where
    K: traits::PublicEncryptionKey<R>,
    R: Role,
{
    public_key: K,
    secret_key: SecretEncryptionKey<R>,
    marker: PhantomData<R>,
}

impl<R, K> EncryptionKeyPair<R, K>
where
    R: Role,
    K: traits::PublicEncryptionKey<R> + Sized,
{
    pub fn new(public_key: K, secret_key: SecretEncryptionKey<R>) -> Self {
        EncryptionKeyPair {
            public_key,
            secret_key,
            marker: PhantomData,
        }
    }

    /// Generate a random key pair to be used for encryption.
    pub fn generate() -> EncryptionKeyPair<R, PublicEncryptionKey<R>> {
        let csprng = thread_rng();

        let secret_key = X25519SecretKey::random_from_rng(csprng);

        let public_key = X25519PublicKey::from(&secret_key);
        let public_key = PublicEncryptionKey::new(public_key);

        let secret_key = SecretEncryptionKey::new(secret_key);

        EncryptionKeyPair::<R, PublicEncryptionKey<R>>::new(public_key, secret_key)
    }

    pub fn public_key(&self) -> &K {
        &self.public_key
    }

    pub fn secret_key(&self) -> &SecretEncryptionKey<R> {
        &self.secret_key
    }

    pub fn raw_public_key(&self) -> X25519PublicKey {
        self.public_key.raw_public_key()
    }
}

impl<R, K> PublicKey for EncryptionKeyPair<R, K>
where
    R: Role,
    K: traits::PublicEncryptionKey<R> + Sized,
{
    fn public_key_hex(&self) -> String {
        hex::encode(self.raw_public_key().as_bytes())
    }
}

pub type SignedEncryptionKeyPair<R> = EncryptionKeyPair<R, SignedPublicEncryptionKey<R>>;
pub type UnsignedEncryptionKeyPair<R> = EncryptionKeyPair<R, PublicEncryptionKey<R>>;

// Specialised functions for the UNSIGNED version of a key pair
impl<R: Role> UnsignedEncryptionKeyPair<R> {
    pub fn from_raw_keys(pk: X25519PublicKey, sk: X25519SecretKey) -> UnsignedEncryptionKeyPair<R> {
        let pk = PublicEncryptionKey::new(pk);
        let sk = SecretEncryptionKey::new(sk);

        UnsignedEncryptionKeyPair::<R>::new(pk, sk)
    }

    pub fn to_signed_key_pair<SR, SK>(
        self,
        signing_key_pair: &SigningKeyPair<SR, SK>,
        not_valid_after: DateTime<Utc>,
    ) -> SignedEncryptionKeyPair<R>
    where
        SR: Role,
        SK: signing::traits::PublicSigningKey<SR>,
    {
        let cert_data =
            KeyCertificateData::new_for_encryption_key(&self.raw_public_key(), not_valid_after);
        let certificate = signing_key_pair.sign(&cert_data);

        let pk =
            SignedPublicEncryptionKey::new(self.public_key().clone(), certificate, not_valid_after);

        SignedEncryptionKeyPair::<R>::new(pk, self.secret_key)
    }

    pub fn to_untrusted(&self) -> UntrustedUnsignedEncryptionKeyPair<R> {
        UntrustedUnsignedEncryptionKeyPair::new(
            self.public_key.to_untrusted(),
            self.secret_key.key.clone(),
        )
    }
}

impl<R: Role> SignedEncryptionKeyPair<R> {
    pub fn to_untrusted(&self) -> UntrustedSignedEncryptionKeyPair<R> {
        UntrustedSignedEncryptionKeyPair::new(
            self.public_key.to_untrusted(),
            self.secret_key.key.clone(),
        )
    }
}
pub mod traits {
    use x25519_dalek::PublicKey as X25519PublicKey;

    use crate::{crypto::keys::role::Role, protocol::constants::X25519_PUBLIC_KEY_LEN};

    pub trait PublicEncryptionKey<T> {
        fn raw_public_key(&self) -> X25519PublicKey;

        fn as_bytes(&self) -> &[u8; X25519_PUBLIC_KEY_LEN];
    }

    impl<T: Role> PublicEncryptionKey<T> for super::PublicEncryptionKey<T> {
        fn raw_public_key(&self) -> X25519PublicKey {
            self.key
        }

        fn as_bytes(&self) -> &[u8; X25519_PUBLIC_KEY_LEN] {
            self.key.as_bytes()
        }
    }

    impl<T: Role> PublicEncryptionKey<T> for super::SignedPublicEncryptionKey<T> {
        fn raw_public_key(&self) -> X25519PublicKey {
            self.key
        }

        fn as_bytes(&self) -> &[u8; X25519_PUBLIC_KEY_LEN] {
            self.key.as_bytes()
        }
    }
}

impl<T, R> traits::PublicEncryptionKey<R> for Box<T>
where
    T: traits::PublicEncryptionKey<R>,
{
    fn raw_public_key(&self) -> X25519PublicKey {
        self.as_ref().raw_public_key()
    }

    fn as_bytes(&self) -> &[u8; X25519_PUBLIC_KEY_LEN] {
        self.as_ref().as_bytes()
    }
}
