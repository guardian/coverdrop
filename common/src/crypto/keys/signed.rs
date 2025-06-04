use chrono::{DateTime, Utc};

use super::{
    encryption::{SignedEncryptionKeyPair, SignedPublicEncryptionKey},
    role::Role,
    signing::{SignedPublicSigningKey, SignedSigningKeyPair},
};

// Strictly speaking the role isn't required here but it helps the type inference
// which reduces the amount of manual typing we need to do on generic functions
pub trait SignedKey<R: Role> {
    fn not_valid_after(&self) -> DateTime<Utc>;

    /// Check if a public key is expired
    fn is_not_valid_after(&self, time: DateTime<Utc>) -> bool {
        self.not_valid_after() < time
    }

    fn as_bytes(&self) -> &[u8];
}

impl<R> SignedKey<R> for SignedEncryptionKeyPair<R>
where
    R: Role,
{
    fn not_valid_after(&self) -> DateTime<Utc> {
        self.public_key().not_valid_after
    }

    fn as_bytes(&self) -> &[u8] {
        self.public_key().as_ref().as_bytes()
    }
}

impl<R> SignedKey<R> for SignedSigningKeyPair<R>
where
    R: Role,
{
    fn not_valid_after(&self) -> DateTime<Utc> {
        self.public_key().not_valid_after
    }

    fn as_bytes(&self) -> &[u8] {
        self.public_key().as_bytes()
    }
}

impl<R> SignedKey<R> for SignedPublicEncryptionKey<R>
where
    R: Role,
{
    fn not_valid_after(&self) -> DateTime<Utc> {
        self.not_valid_after
    }

    fn as_bytes(&self) -> &[u8] {
        self.as_ref().as_bytes()
    }
}

impl<R> SignedKey<R> for SignedPublicSigningKey<R>
where
    R: Role,
{
    fn not_valid_after(&self) -> DateTime<Utc> {
        self.not_valid_after
    }

    fn as_bytes(&self) -> &[u8] {
        self.as_ref().as_bytes()
    }
}
