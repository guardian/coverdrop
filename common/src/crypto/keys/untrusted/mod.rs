use thiserror::Error;

pub mod encryption;
pub mod signing;

pub use super::KeyVerification;

#[derive(Error, Debug)]
pub enum UntrustedKeyError {
    #[error("Certificate is not valid")]
    CertificateNotValid,
    #[error("Certificate has expired")]
    CertificateExpired,
    #[error("Parent key not found")]
    ParentKeyNotFound,
}
