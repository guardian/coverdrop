use axum::response::{IntoResponse, Response};
use axum::{http::StatusCode, Json};
use base64::DecodeError;
use common::api::models::journalist_id::JournalistIdentity;
use common::crypto::keys::untrusted::UntrustedKeyError;
use hex::FromHexError;
use serde_json::json;
use thiserror::Error;

/// Represents an error that happens during setup and
/// does not need to be mapped to an HTTP response
#[derive(Error, Debug)]
pub enum SetupError {
    #[error("failed to parse the database path into a string")]
    ParsingDatabasePathFailed,
}

/// Represents a runtime error that needs to be mapped
/// to an HTTP response
#[derive(Error, Debug)]
pub enum AppError {
    #[error("error")]
    Anyhow(#[from] anyhow::Error),
    #[error("bad message size")]
    BadMessageSize,
    #[error("database error")]
    DatabaseError(#[from] sqlx::Error),
    #[error("loading keys failed {0}")]
    LoadingKeysFailed(String),
    #[error("parsing untrusted key failed")]
    UntrustedKeyError(#[from] UntrustedKeyError),
    #[error("failed to parse hex")]
    HexParseError(#[from] FromHexError),
    #[error("failed to parse hex")]
    Base64Decode(#[from] DecodeError),
    #[error("signature verification failed")]
    SignatureVerificationFailed,
    #[error("client unauthorized to update record")]
    JournalistUnauthorized,
    #[error("journalist id does not match signing key for journalist id: {0}")]
    JournalistIdDoesNotMatchSigningKey(JournalistIdentity),
    #[error("journalist id not found")]
    JournalistIdNotFound,
    #[error("journalist description too long")]
    JournalistDescriptionTooLong,
    #[error("CoverDrop common error: {0}")]
    CommonError(#[from] common::Error),
    #[error("Form signing key not found")]
    SigningKeyNotFound,
    #[error("Failed to make s3 presigned url")]
    S3PresignedUrlError,
    #[error("No organization keys")]
    NoOrganizationKeys,
    #[error("Environment variable not found: {0}")]
    EnvVariableNotFound(&'static str),
    #[error("key has been uploaded too recently")]
    KeyRotationTooRecent,
    #[error("failed to put message on kinesis stream")]
    KinesisStreamPut,
    #[error("backup data not found for journalist id: {0}")]
    BackupDataNotFound(JournalistIdentity),
    #[error("Incorrect Stage found: {0}")]
    IncorrectStageFound(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, err_msg): (StatusCode, String) = match self {
            Self::BadMessageSize => (StatusCode::BAD_REQUEST, "bad message size".into()),
            Self::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "database error".into()),
            Self::LoadingKeysFailed(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "loading keys from disk failed".into(),
            ),
            Self::UntrustedKeyError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to convert untrusted keys to verified keys".into(),
            ),
            Self::HexParseError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to parse hex".into(),
            ),
            Self::SignatureVerificationFailed => (
                StatusCode::UNAUTHORIZED,
                "signature verification failed".into(),
            ),
            Self::JournalistUnauthorized => (
                StatusCode::UNAUTHORIZED,
                "client unauthorized to update record".into(),
            ),
            Self::JournalistIdNotFound => {
                (StatusCode::NOT_FOUND, "journalist id key not found".into())
            }
            Self::JournalistDescriptionTooLong => (
                StatusCode::BAD_REQUEST,
                "journalist description too long".into(),
            ),
            Self::Base64Decode(_) => (StatusCode::BAD_REQUEST, "Failed to decode base64".into()),
            Self::CommonError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".into(),
            ),
            Self::Anyhow(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".into(),
            ),
            Self::SigningKeyNotFound => (
                StatusCode::NOT_FOUND,
                "Signing key not found in API database, it is either expired or never existed"
                    .into(),
            ),
            Self::NoOrganizationKeys => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Organization keys not found".into(),
            ),
            Self::EnvVariableNotFound(var) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Environment variable not found: {var}"),
            ),
            Self::KeyRotationTooRecent => (
                StatusCode::BAD_REQUEST,
                "key has been rotated too recently".into(),
            ),
            Self::KinesisStreamPut => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to put message onto stream".into(),
            ),
            Self::BackupDataNotFound(_) => (StatusCode::NOT_FOUND, "No backup data found".into()),
            Self::JournalistIdDoesNotMatchSigningKey(_) => (
                StatusCode::BAD_REQUEST,
                "The provided journalist id does not match the signing key for journalist id"
                    .into(),
            ),
            Self::S3PresignedUrlError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to make presigned upload url".into(),
            ),
            Self::IncorrectStageFound(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Incorrect Stage found on server".into(),
            ),
        };

        tracing::error!("Error from API: {:?}", self);

        let body = Json(json!({
            "error": err_msg,
        }));

        (status, body).into_response()
    }
}
