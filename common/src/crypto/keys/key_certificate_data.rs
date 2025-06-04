use std::mem::size_of;

use chrono::{DateTime, Utc};

use crate::protocol::constants::X25519_PUBLIC_KEY_LEN;

use super::{Ed25519PublicKey, X25519PublicKey};

const PUBLIC_KEY_AND_NOT_VALID_AFTER_LEN: usize = X25519_PUBLIC_KEY_LEN + size_of::<i64>();

/// A structure used for signing keys and their expiry time.
///
/// This object is very short lived, only long enough to create or verify a signature.
#[derive(Clone, Debug)]
pub struct KeyCertificateData(pub [u8; PUBLIC_KEY_AND_NOT_VALID_AFTER_LEN]);

impl KeyCertificateData {
    fn new_for_key_bytes(key_bytes: &[u8], not_valid_after: DateTime<Utc>) -> Self {
        let mut buf = [0; PUBLIC_KEY_AND_NOT_VALID_AFTER_LEN];
        buf[..X25519_PUBLIC_KEY_LEN].copy_from_slice(key_bytes);

        // Use network endianess since we have to pick a cross-platform representation
        let not_valid_after_secs = not_valid_after.timestamp().to_be_bytes();
        buf[X25519_PUBLIC_KEY_LEN..].copy_from_slice(&not_valid_after_secs);

        KeyCertificateData(buf)
    }

    pub fn new_for_encryption_key(key: &X25519PublicKey, not_valid_after: DateTime<Utc>) -> Self {
        Self::new_for_key_bytes(key.as_bytes(), not_valid_after)
    }

    pub fn new_for_signing_key(key: &Ed25519PublicKey, timestamp: DateTime<Utc>) -> Self {
        Self::new_for_key_bytes(key.as_bytes(), timestamp)
    }
}
