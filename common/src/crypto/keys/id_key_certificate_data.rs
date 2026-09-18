use chrono::{DateTime, Utc};

use crate::{api::models::identity::Identity, protocol::constants::ED25519_PUBLIC_KEY_LEN};

use super::{role::Role, Ed25519PublicKey};

const ID_KEY_CERT_DATA_TAG: &[u8; 16] = b"ID_KEY_CERT_DATA";

/// Extends [`KeyCertificateData`] signing over an additional identity field.
/// This binds the identity key to the journalist, sentinel, or covernode id that owns it.
///
/// Layout (length-prefixed for variable fields):
/// ```text
/// [
///     "ID_KEY_CERT_DATA" (16B) |
///     key (32B) |
///     not_valid_after_be_i64 (8B) |
///     role_len_be_u32 (4B) | role_name |
///     id_len_be_u32 (4B) | id_utf8
/// ]
/// ```
///
/// [`KeyCertificateData`]: super::key_certificate_data::KeyCertificateData
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdKeyCertificateData(Vec<u8>);

impl IdKeyCertificateData {
    pub fn new<R: Role<CertData = IdKeyCertificateData>>(
        key: &Ed25519PublicKey,
        not_valid_after: DateTime<Utc>,
        identity: &impl Identity,
    ) -> Self {
        let role_name_bytes = R::entity_name().as_bytes();
        let id_bytes = identity.as_ref().as_bytes();

        let total_len = ID_KEY_CERT_DATA_TAG.len()
            + ED25519_PUBLIC_KEY_LEN
            + size_of::<i64>() // not_valid_after timestamp
            + size_of::<u32>() // role length prefix
            + role_name_bytes.len()
            + size_of::<u32>() // id length prefix
            + id_bytes.len();
        let mut buf = Vec::with_capacity(total_len);

        buf.extend_from_slice(ID_KEY_CERT_DATA_TAG);
        buf.extend_from_slice(key.as_bytes());
        buf.extend_from_slice(&not_valid_after.timestamp().to_be_bytes());
        // NOTE: length prefixes prevent length-extension attacks
        buf.extend_from_slice(&(role_name_bytes.len() as u32).to_be_bytes());
        buf.extend_from_slice(role_name_bytes);
        buf.extend_from_slice(&(id_bytes.len() as u32).to_be_bytes());
        buf.extend_from_slice(id_bytes);

        IdKeyCertificateData(buf)
    }
}

impl crate::crypto::signable::Signable for IdKeyCertificateData {
    fn as_signable_bytes(&self) -> &[u8] {
        &self.0
    }
}
