use axum::response::{IntoResponse, Response};
use axum::{http::StatusCode, Json};
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
pub enum DeliveryServiceError {
    #[error("error")]
    Anyhow(#[from] anyhow::Error),
    #[error("database error")]
    DatabaseError(#[from] sqlx::Error),
    #[error("common error: {0}")]
    CommonError(#[from] common::Error),
    #[error("not found: {0}")]
    ClientNotFound(String),
    #[error("key packages depleted: {0}")]
    KeyPackagesDepleted(String),
    #[error("missing key packages")]
    MissingKeyPackages,
    #[error("stale epoch: existing epoch {existing} is greater than new epoch {new}")]
    StaleEpoch { existing: i64, new: i64 },
    #[error("epoch race condition: another transaction modified the group concurrently")]
    EpochRaceCondition,
    #[error("malformed MLS message: {0}")]
    MalformedMlsMessage(String),
    #[error("MLS error: {0}")]
    MlsError(String),
    #[error("Form signing key not found")]
    SigningKeyNotFound,
    #[error("signature verification failed")]
    SignatureVerificationFailed,
    #[error("deserialization error: {0}")]
    DeserializationError(String),
}

impl IntoResponse for DeliveryServiceError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            DeliveryServiceError::DatabaseError(ref err) => {
                tracing::error!("Database error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
            DeliveryServiceError::Anyhow(ref err) => {
                tracing::error!("Internal error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal error".to_string(),
                )
            }
            DeliveryServiceError::CommonError(ref err) => {
                tracing::error!("Common error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal error".to_string(),
                )
            }
            DeliveryServiceError::ClientNotFound(ref msg) => {
                tracing::warn!("Not found: {}", msg);
                (StatusCode::NOT_FOUND, msg.clone())
            }
            DeliveryServiceError::KeyPackagesDepleted(ref msg) => {
                tracing::warn!("Key packages depleted: {}", msg);
                (StatusCode::CONFLICT, msg.clone())
            }
            DeliveryServiceError::MissingKeyPackages => {
                tracing::warn!("Missing key packages in request");
                (
                    StatusCode::BAD_REQUEST,
                    "Missing key packages in request".to_string(),
                )
            }
            DeliveryServiceError::StaleEpoch { existing, new } => {
                let msg = format!(
                    "Stale epoch: existing epoch {} is greater than new epoch {}",
                    existing, new
                );
                tracing::warn!("{}", msg);
                (StatusCode::CONFLICT, msg)
            }
            DeliveryServiceError::EpochRaceCondition => {
                let msg =
                    "Epoch race condition: another transaction modified the group concurrently"
                        .to_string();
                tracing::warn!("{}", msg);
                (StatusCode::CONFLICT, msg)
            }
            DeliveryServiceError::MalformedMlsMessage(ref msg) => {
                tracing::warn!("Received malformed MLS message: {}", msg);
                (
                    StatusCode::BAD_REQUEST,
                    format!("Malformed MLS message: {}", msg),
                )
            }
            DeliveryServiceError::MlsError(ref msg) => {
                tracing::error!("MLS error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
            DeliveryServiceError::SigningKeyNotFound => {
                tracing::warn!("Form signing key not found");
                (
                    StatusCode::BAD_REQUEST,
                    "Form signing key not found".to_string(),
                )
            }
            Self::SignatureVerificationFailed => (
                StatusCode::UNAUTHORIZED,
                "signature verification failed".into(),
            ),
            DeliveryServiceError::DeserializationError(ref msg) => {
                tracing::warn!("Deserialisation error: {}", msg);
                (
                    StatusCode::BAD_REQUEST,
                    format!("Deserialisation error: {}", msg),
                )
            }
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}
