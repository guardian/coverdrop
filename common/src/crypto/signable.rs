use ed25519_dalek::VerifyingKey as Ed25519PublicKey;
use x25519_dalek::PublicKey as X25519PublicKey;

use crate::api::models::journalist_id::JournalistIdentity;

use super::keys::key_certificate_data::KeyCertificateData;

pub trait Signable {
    fn as_signable_bytes(&self) -> &[u8];
}

impl Signable for Ed25519PublicKey {
    fn as_signable_bytes(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl Signable for X25519PublicKey {
    fn as_signable_bytes(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl Signable for Vec<u8> {
    fn as_signable_bytes(&self) -> &[u8] {
        self.as_slice()
    }
}

impl Signable for KeyCertificateData {
    fn as_signable_bytes(&self) -> &[u8] {
        &self.0[..]
    }
}

impl Signable for JournalistIdentity {
    fn as_signable_bytes(&self) -> &[u8] {
        self.as_ref().as_bytes()
    }
}
