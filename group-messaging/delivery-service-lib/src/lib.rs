use openmls::prelude::{Ciphersuite, ProtocolVersion};

pub mod client;
pub mod forms;
pub mod models;
pub mod tls_serialized;

/// It's important that the signature algorithm in the ciphersuite is Ed25519, in order to match
/// journalist / sentinel identity [`SignedPublicSigningKey`]s
pub const MLS_CIPHERSUITE: Ciphersuite = Ciphersuite::MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519;

pub const PROTOCOL_VERSION: ProtocolVersion = ProtocolVersion::Mls10;

/// Maximum size in bytes for a TLS-serialized MLS message sent via the delivery service.
/// This limit is enforced on both the client (before sending) and the server (before storing).
pub const MAX_MESSAGE_SIZE_BYTES: usize = 16 * 1024;
