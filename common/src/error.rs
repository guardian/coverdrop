use std::io;

use chacha20poly1305::aead;
use thiserror::Error;

use crate::api::models::journalist_id::JournalistIdentity;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    General(String),
    #[error("IO error: {0}")]
    IO(#[from] io::Error),
    #[error("Error while serializing JSON")]
    JsonSerialization(#[from] serde_json::Error),
    #[error("Error during encryption/decryption")]
    Aead(#[from] aead::Error),
    #[error("Compressed string too long")]
    CompressedStringTooLong(f32),
    #[error("Decompression ratio is too high")]
    DecompressionRatioTooHigh,
    #[error("Invalid padded compressed string")]
    InvalidPaddedCompressedString,
    #[error("Wrong key type")]
    InvalidKeyType,
    #[error("Byte slice wrong size")]
    ByteSliceToOwned,
    #[error("Failed to decrypt")]
    FailedToDecrypt,
    #[error("Failed to encrypt")]
    FailedToEncrypt,
    #[error("Signature error")]
    Signature(#[from] ed25519_dalek::SignatureError),
    #[error("API error {0}: {1}")]
    Api(reqwest::StatusCode, String),
    #[error("Invalid public key hex")]
    InvalidPublicKeyHex,
    #[error("Argon2 salt parse")]
    Argon2SaltParse,
    #[error("Argon2 hash failed")]
    Argon2Hashing,
    #[error("Argon2 hash missing")]
    Argon2Missing,
    #[error("Argon2 bad parameters")]
    Argon2BadParameters,
    #[error("Journalist ID is invalid")]
    InvalidJournalistId,
    #[error("Journalist '{0}' not found")]
    JournalistNotFound(JournalistIdentity),
    #[error("Journalist '{0}' messaging key not found")]
    JournalistMessagingKeyNotFound(JournalistIdentity),
    #[error("CoverNode messaging key not found")]
    CoverNodeMessagingKeyNotFound,
    #[error("Unable to verify key")]
    UnableToVerifyKey,
    #[error("Secret key bad size")]
    SecretKeyBadSize,
    #[error("CoverNode ID is invalid, must be in the form '{expected_pattern}'")]
    InvalidCoverNodeId { expected_pattern: &'static str },
    #[error("Invalid key")]
    InvalidKey,
    #[error("{0} key pair not found")]
    LatestKeyPairNotFound(&'static str),
    #[error("Signing key has expired")]
    SigningKeyExpired,
    #[error("Invalid X25519 public key bytes")]
    InvalidPublicKeyBytes,
    #[error("Content exceeds maximum length: {0}")]
    PaddedContentTooLarge(u64),
    #[error("Padded byte array {0} has not enough size, needs least {1}")]
    PaddedByteVectorNotEnoughSpace(u64, u64),
    #[error("Invalid padded byte array")]
    PaddedByteArrayInvalid,
    #[error("Stepping padded byte vector is not a multiple of the expected step size")]
    PaddedByteVectorNotMultipleOfStepSize,
    #[error("Failed to split secret into shares: {0}")]
    SecretSharingSplitError(String),
    #[error("Too few secret shares provided; at least {0} required, but got {1}")]
    SecretSharingTooFewShares(u64, u64),
    #[error("Share size mismatch; expected {0} bytes, but got {1}")]
    SecretSharingShareSizeMismatch(usize, usize),
}
