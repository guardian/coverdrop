use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("Public key not found")]
    PublicKeyNotFound,
    #[error("Multiple keys found for search")]
    MultiplePublicKeys,
}
