pub mod encryption;
pub mod key_certificate_data;
pub mod public_key;
pub mod role;
pub mod serde;
pub mod signed;
pub mod signing;
pub mod untrusted;

pub type Ed25519PublicKey = ed25519_dalek::VerifyingKey;
pub type Ed25519SigningKeyPair = ed25519_dalek::SigningKey;
pub type Ed25519Signature = ed25519_dalek::Signature;

pub type X25519PublicKey = x25519_dalek::PublicKey;
pub type X25519SecretKey = x25519_dalek::StaticSecret;
