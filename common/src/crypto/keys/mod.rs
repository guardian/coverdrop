pub mod encryption;
pub mod id_key_certificate_data;
pub mod key_certificate_data;
pub mod public_key;
pub mod role;
pub mod serde;
pub mod signed;
pub mod signing;
pub mod untrusted;

/// Strategy used to verify a public key against its parent signing key.
///
/// - `STRICT_REQUIRE_SIGNATURE`: Enforces that a valid signature is present and verifies it,
///   as well as verifying the legacy certificate if present. Used for covernode and journalist identity keys.
/// - `ACCEPT_LEGACY_CERTIFICATE`: Verifies the legacy certificate and also verifies the
///   signature if it is present. Used for organization, provisioning, and messaging keys.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum KeyVerification {
    STRICT_REQUIRE_SIGNATURE,
    ACCEPT_LEGACY_CERTIFICATE,
}

pub type Ed25519PublicKey = ed25519_dalek::VerifyingKey;
pub type Ed25519SigningKeyPair = ed25519_dalek::SigningKey;
pub type Ed25519Signature = ed25519_dalek::Signature;

pub type X25519PublicKey = x25519_dalek::PublicKey;
pub type X25519SecretKey = x25519_dalek::StaticSecret;
