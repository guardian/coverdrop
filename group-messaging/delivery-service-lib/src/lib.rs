use openmls::prelude::{Ciphersuite, ProtocolVersion};

pub mod client;
pub mod forms;
pub mod models;
pub mod tls_serialized;

/// It's important that the signature algorithm in the ciphersuite is Ed25519, in order to match
/// journalist / sentinel identity [`SignedPublicSigningKey`]s
pub const MLS_CIPHERSUITE: Ciphersuite = Ciphersuite::MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519;

pub const PROTOCOL_VERSION: ProtocolVersion = ProtocolVersion::Mls10;
