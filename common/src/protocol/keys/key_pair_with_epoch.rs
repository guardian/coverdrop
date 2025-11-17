use std::marker::PhantomData;

use crate::{
    crypto::{
        keys::{
            role::Role, signed::SignedKey, signing::traits::PublicSigningKey, Ed25519PublicKey,
        },
        Signable, Signature,
    },
    epoch::Epoch,
    protocol::{
        constants::ED25519_PUBLIC_KEY_LEN,
        keys::{
            CoverNodeIdKeyPair, CoverNodeMessagingKeyPair, UntrustedCoverNodeIdKeyPair,
            UntrustedCoverNodeMessagingKeyPair,
        },
        roles::{CoverNodeId, CoverNodeMessaging},
    },
    Error,
};
use chrono::{DateTime, Utc};

//
// Untrusted version, cannot be used for cryptography
//
#[derive(Clone)]
pub struct UntrustedKeyPairWithEpoch<R, T>
where
    R: Role,
{
    pub key_pair: T,
    pub epoch: Epoch,
    pub created_at: DateTime<Utc>,
    marker: PhantomData<R>,
}

impl<R, T> UntrustedKeyPairWithEpoch<R, T>
where
    R: Role,
{
    pub fn new(key_pair: T, epoch: Epoch, created_at: DateTime<Utc>) -> Self {
        Self {
            key_pair,
            epoch,
            created_at,
            marker: PhantomData,
        }
    }
}

pub type UntrustedCoverNodeMessagingKeyPairWithEpoch =
    UntrustedKeyPairWithEpoch<CoverNodeMessaging, UntrustedCoverNodeMessagingKeyPair>;
pub type UntrustedCoverNodeIdKeyPairWithEpoch =
    UntrustedKeyPairWithEpoch<CoverNodeId, UntrustedCoverNodeIdKeyPair>;

//
// Trusted version, can be used for cryptography
//

#[derive(Clone)]
pub struct KeyPairWithEpoch<R, T>
where
    R: Role,
{
    pub key_pair: T,
    pub epoch: Epoch,
    pub created_at: DateTime<Utc>,
    marker: PhantomData<R>,
}

impl<R, T> KeyPairWithEpoch<R, T>
where
    R: Role,
    T: SignedKey<R>,
{
    pub fn new(key_pair: T, epoch: Epoch, created_at: DateTime<Utc>) -> Self {
        Self {
            key_pair,
            epoch,
            created_at,
            marker: PhantomData,
        }
    }
}

// Forward the PublicSigningKey trait to the underlying key
// this allows us to avoid a lot of mapping the epoch away
impl<R, T> PublicSigningKey<R> for KeyPairWithEpoch<R, T>
where
    R: Role,
    T: PublicSigningKey<R> + Clone,
{
    fn verify<S>(
        &self,
        message: &S,
        signature: &Signature<S>,
        now: DateTime<Utc>,
    ) -> Result<(), Error>
    where
        S: Signable,
    {
        self.key_pair.verify(message, signature, now)
    }

    fn raw_public_key(&self) -> Ed25519PublicKey {
        self.key_pair.raw_public_key()
    }

    fn as_bytes(&self) -> &[u8; ED25519_PUBLIC_KEY_LEN] {
        PublicSigningKey::as_bytes(&self.key_pair)
    }
}

impl<R, T> SignedKey<R> for KeyPairWithEpoch<R, T>
where
    R: Role,
    T: SignedKey<R>,
{
    fn not_valid_after(&self) -> DateTime<Utc> {
        self.key_pair.not_valid_after()
    }

    fn as_bytes(&self) -> &[u8] {
        SignedKey::as_bytes(&self.key_pair)
    }
}

pub type CoverNodeMessagingKeyPairWithEpoch =
    KeyPairWithEpoch<CoverNodeMessaging, CoverNodeMessagingKeyPair>;
pub type CoverNodeIdKeyPairWithEpoch = KeyPairWithEpoch<CoverNodeId, CoverNodeIdKeyPair>;
