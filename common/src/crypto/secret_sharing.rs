use crate::crypto::Encryptable;
use crate::Error;
use rand::{thread_rng, RngCore};
use std::fmt::Debug;
use zeroize::Zeroize;

/// The size of the secret in bytes for secret sharing (32 bytes = 256 bits) is sufficiently large
/// for our 128-bit security level.
pub const SECRET_SHARING_SECRET_SIZE: usize = 32;

#[derive(Zeroize, Eq, PartialEq)]
pub struct SecretSharingSecret([u8; SECRET_SHARING_SECRET_SIZE]);

impl SecretSharingSecret {
    /// Creates a new random `SecretSharingSecret`
    pub fn generate() -> Result<Self, Error> {
        let mut res = SecretSharingSecret([0u8; SECRET_SHARING_SECRET_SIZE]);
        thread_rng().fill_bytes(&mut res.0);
        Ok(res)
    }

    /// Returns the secret as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Clone, Zeroize, Eq, PartialEq)]
pub struct SecretSharingShare(Vec<u8>);

impl Encryptable for SecretSharingShare {
    fn as_unencrypted_bytes(&self) -> &[u8] {
        &self.0
    }

    fn from_unencrypted_bytes(bytes: Vec<u8>) -> Result<Self, Error> {
        Ok(SecretSharingShare(bytes))
    }
}

impl Debug for SecretSharingShare {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecretSharingShare")
            .field("len", &self.0.len())
            .finish()
    }
}

/// The trait for secret sharing schemes. It defines the methods to split a secret into shares and
/// reconstruct the secret from those shares. The `K` type parameter defines the minimum number of
/// shares required to reconstruct the secret.
pub trait SecretSharingScheme<const K: usize> {
    /// Splits the secret into `N` shares, of which at least `K` are required to reconstruct the secret.
    fn split(secret: SecretSharingSecret, n: usize) -> Result<Vec<SecretSharingShare>, Error>;

    /// Reconstructs the secret from `shares`, which must contain at least `K` shares.
    fn combine(shares: [SecretSharingShare; K]) -> Result<SecretSharingSecret, Error>;
}

/// A simple secret sharing implementation where a single share is sufficient to reconstruct the
/// secret, i.e. `K = 1`. All shares are identical to the original secret.
pub struct SingleShareSecretSharing;

impl SecretSharingScheme<1> for SingleShareSecretSharing {
    fn split(secret: SecretSharingSecret, n: usize) -> Result<Vec<SecretSharingShare>, Error> {
        let shares = vec![SecretSharingShare(secret.0.to_vec()); n];
        Ok(shares)
    }

    fn combine(shares: [SecretSharingShare; 1]) -> Result<SecretSharingSecret, Error> {
        let only_share = &shares[0].0;
        Ok(SecretSharingSecret(
            only_share.as_slice().try_into().map_err(|_| {
                Error::SecretSharingShareSizeMismatch(SECRET_SHARING_SECRET_SIZE, only_share.len())
            })?,
        ))
    }
}

/// A placeholder for the general secret sharing implementation. This is currently not implemented
/// and will throw an error if used. It is intended to be replaced with a proper implementation
/// in the future once we have picked a suitable library.
struct GeneralSecretSharing;

impl<const K: usize> SecretSharingScheme<K> for GeneralSecretSharing {
    fn split(_secret: SecretSharingSecret, _n: usize) -> Result<Vec<SecretSharingShare>, Error> {
        todo!("Not yet implemented: https://github.com/guardian/coverdrop-internal/issues/3451")
    }

    fn combine(_shares: [SecretSharingShare; K]) -> Result<SecretSharingSecret, Error> {
        todo!("Not yet implemented: https://github.com/guardian/coverdrop-internal/issues/3451")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_sharing_secret_is_random() {
        let secret1 = SecretSharingSecret::generate().unwrap();
        let secret2 = SecretSharingSecret::generate().unwrap();
        assert_ne!(secret1.as_bytes(), secret2.as_bytes());
    }
}
