use std::env;

use crate::{
    cache_control::{add_cache_control_header, HEALTHCHECK_TTL_IN_SECONDS, STATUS_TTL_IN_SECONDS},
    error::AppError,
    services::database::Database,
};
use axum::extract::State;
use axum::Json;
use chrono::Duration;
use common::{
    api::{
        forms::PostSystemStatusEventForm,
        models::general::{PublishedStatusEvent, StatusEvent},
    },
    aws::ses::client::{SendEmailConfig, SesClient},
    healthcheck::HealthCheck,
    system::forms::PostLogConfigForm,
    time,
    tracing::TracingReloadHandle,
};
use http::HeaderMap;

fn get_env_or_error(var: &'static str) -> Result<String, AppError> {
    env::var(var).map_err(|_| AppError::EnvVariableNotFound(var))
}

pub async fn get_healthcheck() -> (HeaderMap, Json<HealthCheck>) {
    let result = HealthCheck::new("api", "ok");

    let mut headers = HeaderMap::new();
    add_cache_control_header(&mut headers, Duration::seconds(HEALTHCHECK_TTL_IN_SECONDS));

    (headers, Json(result))
}

pub async fn get_latest_status(
    State(db): State<Database>,
) -> Result<(HeaderMap, Json<PublishedStatusEvent>), AppError> {
    let status = db.system_queries.get_latest_status().await?;

    let mut headers = HeaderMap::new();
    add_cache_control_header(&mut headers, Duration::seconds(STATUS_TTL_IN_SECONDS));

    match status {
        Some(s) => Ok((headers, Json(s.into_published()))),
        None => Ok((
            headers,
            Json(StatusEvent::no_information(time::now()).into_published()),
        )),
    }
}

pub async fn post_status_event(
    State(db): State<Database>,
    Json(body): Json<PostSystemStatusEventForm>,
) -> Result<(), AppError> {
    let admin_pk = db
        .system_key_queries
        .find_admin_pk_from_ed25519_pk(body.signing_pk(), time::now())
        .await?
        .ok_or(AppError::SigningKeyNotFound)?;

    let Ok(body) = body.to_verified_form_data(&admin_pk, time::now()) else {
        return Err(AppError::SignatureVerificationFailed);
    };

    db.system_queries.insert_status_event(&body).await?;

    tracing::info!("CoverDrop Status updated to {:?}", &body.status.status);

    // All the environment variables below are fetched from the systemd environment file
    let in_aws = get_env_or_error("IN_AWS");

    // Check if we are in AWS, so the API can send an email to the team
    // when a status event is posted.
    // If the variable `IN_AWS` does not exist, it means the API is running locally
    // therefore we return here, as no email needs to be sent out
    if in_aws.is_err() {
        tracing::debug!(
            "Environment variable IN_AWS not found. Assuming the API is running locally. No status update email will be sent."
        );
        return Ok(());
    }

    let region = get_env_or_error("AWS_REGION")?;
    let stage = get_env_or_error("STAGE")?;
    let email_identity_domain = get_env_or_error("EMAIL_IDENTITY_DOMAIN")?;
    let team_email_address = get_env_or_error("TEAM_EMAIL_ADDRESS")?;

    let ses_client = SesClient::new(
        region,
        None,
        format!("CoverDrop {stage} <alerts@{email_identity_domain}>"),
    )
    .await;

    let email_config = SendEmailConfig {
        to: team_email_address.clone(),
        subject: "🔔 CoverDrop system status updated".into(),
        reply_to: team_email_address,
        body: format!(
            "The system status has just been updated to:\n{}",
            serde_json::to_string_pretty(&body.status).unwrap()
        ),
    };
    tracing::debug!("Sending email: {:?}", email_config);

    match ses_client.send_email(email_config).await {
        Ok(_) => tracing::debug!("Status update email sent"),
        Err(e) => tracing::debug!("Status update email error: {}", e),
    };

    Ok(())
}

pub async fn post_reload_tracing(
    State(db): State<Database>,
    State(tracing_reload_handle): State<TracingReloadHandle>,
    Json(form): Json<PostLogConfigForm>,
) -> Result<(), AppError> {
    let admin_pk = db
        .system_key_queries
        .find_admin_pk_from_ed25519_pk(form.signing_pk(), time::now())
        .await?
        .ok_or(AppError::SigningKeyNotFound)?;

    let Ok(body) = form.to_verified_form_data(&admin_pk, time::now()) else {
        return Err(AppError::SignatureVerificationFailed);
    };

    tracing_reload_handle
        .update(&body.rust_log_directive)
        .map_err(|e| {
            tracing::error!("Failed to update log config: {:?}", e);
            AppError::Anyhow(e)
        })
}
