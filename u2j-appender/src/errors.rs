use aws_sdk_kinesis::{
    config::http::HttpResponse, error::SdkError, operation::put_record::PutRecordError,
};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

pub type KinesisPutRecordError = SdkError<PutRecordError, HttpResponse>;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("json parse error")]
    JsonParse(#[from] serde_json::Error),
    #[error("append back pressure")]
    AppendBackPressure(KinesisPutRecordError),
    #[error("append failed")]
    AppendFailed(KinesisPutRecordError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, err_msg): (StatusCode, &'static str) = match self {
            Self::JsonParse(_) => (StatusCode::BAD_REQUEST, "Failed to parse json"),
            Self::AppendBackPressure(_) => (StatusCode::TOO_MANY_REQUESTS, "Too many requests"),
            Self::AppendFailed(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unexpected append failure",
            ),
        };

        tracing::error!("Error in appender: {:?}", self);

        let body = Json(json!({
            "error": err_msg,
        }));

        (status, body).into_response()
    }
}
