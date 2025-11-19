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

/// Type representing the number of shares in the secret sharing schemes. Used for both the total
/// number of shares `n` and the threshold number of shares `k`.
pub type ShareCount = u8;

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
/// reconstruct the secret from those shares.
pub trait SecretSharingScheme {
    /// Splits the secret into `n` shares, of which at least `k` are required to reconstruct the secret.
    fn split(
        secret: SecretSharingSecret,
        k: ShareCount,
        n: ShareCount,
    ) -> Result<Vec<SecretSharingShare>, Error>;

    /// Reconstructs the secret from `shares`, which must contain at least `k` shares. Note that the
    /// parameter `k` is provided to allow validation of the number of shares, but is not strictly
    /// necessary since the implementations will try to combine shares opportunistically.
    fn combine(
        shares: Vec<SecretSharingShare>,
        k: ShareCount,
    ) -> Result<SecretSharingSecret, Error>;
}

/// A simple secret sharing implementation where a single share is sufficient to reconstruct the
/// secret, i.e. `k = 1`. All shares are identical to the original secret.
pub struct SingleShareSecretSharing;

impl SecretSharingScheme for SingleShareSecretSharing {
    fn split(
        secret: SecretSharingSecret,
        k: ShareCount,
        n: ShareCount,
    ) -> Result<Vec<SecretSharingShare>, Error> {
        if k != 1 {
            return Err(Error::SecretSharingSplitError(format!(
                "SingleShareSecretSharing only supports k=1, but got k={}",
                k
            )));
        }
        let shares = vec![SecretSharingShare(secret.0.to_vec()); n as usize];
        Ok(shares)
    }

    fn combine(
        shares: Vec<SecretSharingShare>,
        k: ShareCount,
    ) -> Result<SecretSharingSecret, Error> {
        if k != 1 {
            return Err(Error::SecretSharingSplitError(format!(
                "SingleShareSecretSharing only supports k=1, but got k={}",
                k
            )));
        }
        if shares.is_empty() {
            return Err(Error::SecretSharingSplitError(
                "No shares provided".to_string(),
            ));
        }
        let only_share = &shares[0].0;
        Ok(SecretSharingSecret(
            only_share.as_slice().try_into().map_err(|_| {
                Error::SecretSharingShareSizeMismatch(SECRET_SHARING_SECRET_SIZE, only_share.len())
            })?,
        ))
    }
}

/// General secret sharing implementation that allows splitting a secret into `n` shares,
/// of which at least `k` are required to reconstruct the secret. We use the `sss-rs` library
/// for Shamir's Secret Sharing scheme: https://github.com/dsprenkels/sss-rs.
///
/// Note: The shamirsecretsharing library requires exactly 64 bytes of data. We pad our 32-byte
/// secrets to 64 bytes before splitting and truncate back to 32 bytes after combining.
pub struct GeneralSecretSharing;

impl SecretSharingScheme for GeneralSecretSharing {
    fn split(
        secret: SecretSharingSecret,
        k: ShareCount,
        n: ShareCount,
    ) -> Result<Vec<SecretSharingShare>, Error> {
        if k < 1 {
            return Err(Error::SecretSharingSplitError(format!(
                "k must be at least 1, but got k={}",
                k
            )));
        }

        if n < k {
            return Err(Error::SecretSharingSplitError(format!(
                "Number of shares n={} must be at least k={}",
                n, k,
            )));
        }

        // Pad the secret to 64 bytes (as required by shamirsecretsharing library)
        let mut padded_secret = [0u8; shamirsecretsharing::DATA_SIZE];
        padded_secret[..SECRET_SHARING_SECRET_SIZE].copy_from_slice(&secret.0);

        let shares = shamirsecretsharing::create_shares(&padded_secret, n, k)
            .map_err(|e| Error::SecretSharingSplitError(e.to_string()))?;

        let secret_shares = shares.into_iter().map(SecretSharingShare).collect();
        Ok(secret_shares)
    }

    fn combine(
        shares: Vec<SecretSharingShare>,
        k: ShareCount,
    ) -> Result<SecretSharingSecret, Error> {
        if k < 1 {
            return Err(Error::SecretSharingSplitError(format!(
                "k must be at least 1, but got k={}",
                k
            )));
        }

        if shares.len() < k as usize {
            return Err(Error::SecretSharingSplitError(format!(
                "Not enough shares provided: got {}, need at least {}",
                shares.len(),
                k
            )));
        }

        let share_bytes: Vec<Vec<u8>> = shares.iter().map(|share| share.0.clone()).collect();

        let restored = shamirsecretsharing::combine_shares(&share_bytes)
            .map_err(|e| Error::SecretSharingSplitError(e.to_string()))?
            .ok_or_else(|| {
                Error::SecretSharingSplitError("Failed to restore secret from shares".to_string())
            })?;

        // The restored data is 64 bytes, but we only need the first 32 bytes
        if restored.len() < SECRET_SHARING_SECRET_SIZE {
            return Err(Error::SecretSharingShareSizeMismatch(
                SECRET_SHARING_SECRET_SIZE,
                restored.len(),
            ));
        }

        let secret_array: [u8; SECRET_SHARING_SECRET_SIZE] = restored[..SECRET_SHARING_SECRET_SIZE]
            .try_into()
            .map_err(|_| {
                Error::SecretSharingShareSizeMismatch(SECRET_SHARING_SECRET_SIZE, restored.len())
            })?;

        Ok(SecretSharingSecret(secret_array))
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

    #[test]
    fn general_secret_sharing_happy_path_2_of_3() {
        let secret = SecretSharingSecret::generate().unwrap();
        let secret_bytes = secret.as_bytes().to_vec();

        // Split into 3 shares, requiring 2 to reconstruct
        let shares = GeneralSecretSharing::split(secret, 2, 3).unwrap();
        assert_eq!(shares.len(), 3);

        // Reconstruct with first two shares
        let reconstructed =
            GeneralSecretSharing::combine(vec![shares[0].clone(), shares[1].clone()], 2).unwrap();
        assert_eq!(reconstructed.as_bytes(), secret_bytes.as_slice());

        // Reconstruct with different pair
        let reconstructed2 =
            GeneralSecretSharing::combine(vec![shares[0].clone(), shares[2].clone()], 2).unwrap();
        assert_eq!(reconstructed2.as_bytes(), secret_bytes.as_slice());

        // Reconstruct with another different pair
        let reconstructed3 =
            GeneralSecretSharing::combine(vec![shares[1].clone(), shares[2].clone()], 2).unwrap();
        assert_eq!(reconstructed3.as_bytes(), secret_bytes.as_slice());
    }

    #[test]
    fn general_secret_sharing_happy_path_3_of_5() {
        let secret = SecretSharingSecret::generate().unwrap();
        let secret_bytes = secret.as_bytes().to_vec();

        // Split into 5 shares, requiring 3 to reconstruct
        let shares = GeneralSecretSharing::split(secret, 3, 5).unwrap();
        assert_eq!(shares.len(), 5);

        // Reconstruct with shares 0, 1, 2
        let reconstructed = GeneralSecretSharing::combine(
            vec![shares[0].clone(), shares[1].clone(), shares[2].clone()],
            3,
        )
        .unwrap();
        assert_eq!(reconstructed.as_bytes(), secret_bytes.as_slice());

        // Reconstruct with shares 2, 3, 4
        let reconstructed2 = GeneralSecretSharing::combine(
            vec![shares[2].clone(), shares[3].clone(), shares[4].clone()],
            3,
        )
        .unwrap();
        assert_eq!(reconstructed2.as_bytes(), secret_bytes.as_slice());

        // Reconstruct with shares 0, 2, 4
        let reconstructed3 = GeneralSecretSharing::combine(
            vec![shares[0].clone(), shares[2].clone(), shares[4].clone()],
            3,
        )
        .unwrap();
        assert_eq!(reconstructed3.as_bytes(), secret_bytes.as_slice());
    }

    #[test]
    fn general_secret_sharing_n_less_than_k() {
        let secret = SecretSharingSecret::generate().unwrap();

        // Try to split with n=2 but k=3 (n < k)
        let result = GeneralSecretSharing::split(secret, 3, 2);

        match result {
            Err(Error::SecretSharingSplitError(msg)) => {
                assert!(msg.contains("Number of shares n=2 must be at least k=3"));
            }
            _ => panic!("Expected SecretSharingSplitError"),
        }
    }

    #[test]
    fn general_secret_sharing_manipulated_share_changes_outcome() {
        let secret = SecretSharingSecret::generate().unwrap();
        let secret_bytes = secret.as_bytes().to_vec();

        // Split into 3 shares, requiring 2 to reconstruct
        let shares = GeneralSecretSharing::split(secret, 2, 3).unwrap();

        // Verify normal reconstruction works
        let reconstructed =
            GeneralSecretSharing::combine(vec![shares[0].clone(), shares[1].clone()], 2).unwrap();
        assert_eq!(reconstructed.as_bytes(), secret_bytes.as_slice());

        // Manipulate the first share by flipping a bit
        let mut manipulated_share = shares[0].clone();
        if !manipulated_share.0.is_empty() {
            manipulated_share.0[0] ^= 0xFF; // Flip all bits in first byte
        }

        // Attempt to reconstruct with manipulated share - this should fail
        let result = GeneralSecretSharing::combine(vec![manipulated_share, shares[1].clone()], 2);
        match result {
            Err(Error::SecretSharingSplitError(msg)) => {
                assert!(msg.contains("Failed to restore secret from shares"));
            }
            _ => panic!("Expected SecretSharingSplitError when using manipulated share"),
        }
    }

    #[test]
    fn general_secret_sharing_mixed_shares_from_different_secrets() {
        let secret1 = SecretSharingSecret::generate().unwrap();
        let secret2 = SecretSharingSecret::generate().unwrap();

        let secret1_bytes = secret1.as_bytes().to_vec();
        let secret2_bytes = secret2.as_bytes().to_vec();
        assert_ne!(secret1_bytes, secret2_bytes);

        // Split both secrets into 3 shares each, requiring 2 to reconstruct
        let shares1 = GeneralSecretSharing::split(secret1, 2, 3).unwrap();
        let shares2 = GeneralSecretSharing::split(secret2, 2, 3).unwrap();

        let reconstructed1 =
            GeneralSecretSharing::combine(vec![shares1[0].clone(), shares1[1].clone()], 2).unwrap();
        assert_eq!(reconstructed1.as_bytes(), secret1_bytes.as_slice());
        let reconstructed2 =
            GeneralSecretSharing::combine(vec![shares2[0].clone(), shares2[1].clone()], 2).unwrap();
        assert_eq!(reconstructed2.as_bytes(), secret2_bytes.as_slice());

        // Mix shares from different secrets - this should fail
        let result = GeneralSecretSharing::combine(vec![shares1[0].clone(), shares2[1].clone()], 2);
        match result {
            Err(Error::SecretSharingSplitError(msg)) => {
                assert!(msg.contains("Failed to restore secret from shares"));
            }
            _ => {
                panic!("Expected SecretSharingSplitError when mixing shares from different secrets")
            }
        }
    }

    #[test]
    fn general_secret_sharing_too_few_shares_provided() {
        let secret = SecretSharingSecret::generate().unwrap();

        // Split into 5 shares, requiring 3 to reconstruct
        let shares = GeneralSecretSharing::split(secret, 3, 5).unwrap();
        assert_eq!(shares.len(), 5);

        // Only provide effectively 2 shares (one is duplicated)
        let reconstructed = GeneralSecretSharing::combine(
            vec![shares[0].clone(), shares[1].clone(), shares[1].clone()],
            3,
        );
        match reconstructed {
            Err(Error::SecretSharingSplitError(msg)) => {
                assert!(msg.contains("Failed to restore secret from shares"));
            }
            _ => panic!("Expected SecretSharingSplitError due to insufficient unique shares"),
        }
    }

    #[test]
    fn general_secret_sharing_k_equals_1() {
        let secret = SecretSharingSecret::generate().unwrap();
        let secret_bytes = secret.as_bytes().to_vec();

        // Split into 5 shares, requiring only 1 to reconstruct
        let shares = GeneralSecretSharing::split(secret, 1, 5).unwrap();
        assert_eq!(shares.len(), 5);

        let reconstructed = GeneralSecretSharing::combine(vec![shares[0].clone()], 1).unwrap();
        assert_eq!(reconstructed.as_bytes(), secret_bytes.as_slice());

        let reconstructed2 = GeneralSecretSharing::combine(vec![shares[3].clone()], 1).unwrap();
        assert_eq!(reconstructed2.as_bytes(), secret_bytes.as_slice());
    }

    #[test]
    fn general_secret_sharing_k_equals_n() {
        let secret = SecretSharingSecret::generate().unwrap();
        let secret_bytes = secret.as_bytes().to_vec();

        // Split into 4 shares, requiring all 4 to reconstruct
        let shares = GeneralSecretSharing::split(secret, 4, 4).unwrap();
        assert_eq!(shares.len(), 4);

        let reconstructed = GeneralSecretSharing::combine(
            vec![
                shares[0].clone(),
                shares[1].clone(),
                shares[2].clone(),
                shares[3].clone(),
            ],
            4,
        )
        .unwrap();
        assert_eq!(reconstructed.as_bytes(), secret_bytes.as_slice());

        // Verify that fewer than k shares fails
        let result = GeneralSecretSharing::combine(
            vec![shares[0].clone(), shares[1].clone(), shares[2].clone()],
            4,
        );
        match result {
            Err(Error::SecretSharingSplitError(msg)) => {
                assert!(msg.contains("Not enough shares provided: got 3, need at least 4"));
            }
            _ => panic!("Expected SecretSharingSplitError when providing fewer than k shares"),
        }
    }

    #[test]
    fn general_secret_sharing_k_greater_than_n() {
        let secret = SecretSharingSecret::generate().unwrap();

        // Try to split with k=5 but n=3 (k > n)
        let result = GeneralSecretSharing::split(secret, 5, 3);

        match result {
            Err(Error::SecretSharingSplitError(msg)) => {
                assert!(msg.contains("Number of shares n=3 must be at least k=5"));
            }
            _ => panic!("Expected SecretSharingSplitError when k > n"),
        }
    }
}
