use axum::response::{IntoResponse, Response};
use axum::{http::StatusCode, Json};
use common::crypto::keys::untrusted::UntrustedKeyError;
use serde_json::json;
use thiserror::Error;

/// Represents a runtime error that needs to be mapped
/// to an HTTP response
#[derive(Error, Debug)]
pub enum AppError {
    #[error("covernode id key rotation form is signed by an unknown covernode or covernode identity key")]
    UnknownCoverNodeIdentityOrCoverNodeIdKey,
    #[error("parsing untrusted key failed")]
    UntrustedKeyError(#[from] UntrustedKeyError),
    #[error("journalist id key rotation form is signed by an unknown journalist")]
    UnknownJournalistId,
    #[error("rotation request signed by valid covernode key, the key already exists but identity does not match")]
    CoverNodeIdRotationIdentityMismatch,
    #[error("rotation request signed by valid journalist key, the key already exists but identity does not match")]
    JournalistIdRotationIdentityMismatch,
    #[error("certificate data used for verification is invalid")]
    CertificateDataVerificationFailed,
    #[error("CoverDrop common error: {0}")]
    CommonError(#[from] common::Error),
    #[error("Error: {0}")]
    GenericError(String),
    #[error("Database error: {0:?}")]
    DatabaseError(#[from] anyhow::Error),
}

impl AppError {
    pub fn from_api_client_error(e: anyhow::Error) -> Self {
        AppError::GenericError(format!("Failure in API client: {e}"))
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, err_msg): (StatusCode, String) = match self {
            AppError::UntrustedKeyError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to convert untrusted keys to verified keys".into(),
            ),
            AppError::CoverNodeIdRotationIdentityMismatch => (
                StatusCode::BAD_REQUEST,
                "rotation request signed by valid covernode key, the key already exists but identity does not match"
                    .into(),
            ),
            AppError::JournalistIdRotationIdentityMismatch => (
                StatusCode::BAD_REQUEST,
                "rotation request signed by valid journalist key, the key already exists but identity does not match"
                    .into(),
            ),
            AppError::CertificateDataVerificationFailed => (
                StatusCode::UNAUTHORIZED,
                "certificate data verification failed".into(),
            ),
            AppError::CommonError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error in common library: {e}"),
            ),
            AppError::GenericError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e),
            AppError::UnknownJournalistId => (StatusCode::NOT_FOUND, "signing journalist not found".into()),
            AppError::UnknownCoverNodeIdentityOrCoverNodeIdKey => (StatusCode::NOT_FOUND, "signing covernode not found".into()),
            AppError::DatabaseError(e) =>  {
                tracing::error!("Error in database: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "error in database".into())
            }
        };

        tracing::error!("Error from Identity API: {}", err_msg);

        let body = Json(json!({
            "error": err_msg,
        }));

        (status, body).into_response()
    }
}
