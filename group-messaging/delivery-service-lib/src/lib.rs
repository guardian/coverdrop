use openmls::prelude::{Ciphersuite, ProtocolVersion};

pub mod client;
pub mod forms;
pub mod models;
pub mod tls_serialized;

/// The use of DH X25519, ChaCha20Poly1305, and Ed25519 matches the cryptographic primitives of
/// the CoverDrop protocol. We're using RustCrypto as a crypto provider so that OpenMLS
/// uses the same underlying crypto libraries as the rest of the project.
/// It's especially important that the signature algorithm in the ciphersuite be Ed25519,
/// in order to match Sentinel Identity [`SignedPublicSigningKey`]s
pub const MLS_CIPHERSUITE: Ciphersuite =
    Ciphersuite::MLS_128_DHKEMX25519_CHACHA20POLY1305_SHA256_Ed25519;

pub const PROTOCOL_VERSION: ProtocolVersion = ProtocolVersion::Mls10;

/// Maximum size in bytes for a TLS-serialized MLS message sent via the delivery service.
/// This limit is enforced on both the client (before sending) and the server (before storing).
pub const MAX_MESSAGE_SIZE_BYTES: usize = 16 * 1024;
