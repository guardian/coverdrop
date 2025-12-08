use crate::anchor_org_pk_cache::AnchorOrganizationPublicKeyCache;
use crate::controllers::general::get_env_or_error;
use crate::error::AppError;
use crate::services::database::Database;
use axum::extract::State;
use axum::Json;
use clap::ValueEnum;
use common::api::forms::{PostBackupIdKeyForm, PostBackupMsgKeyForm};
use common::aws::s3::client::S3Client;
use common::backup::forms::retrieve_upload_url::RetrieveUploadUrlForm;
use common::backup::keys::{verify_backup_id_pk, verify_backup_msg_pk};
use common::clap::Stage;
use common::protocol::backup::get_backup_bucket_name;
use common::protocol::constants::HOUR_IN_SECONDS;
use common::time;

pub async fn retrieve_upload_url(
    State(anchor_org_pks): State<AnchorOrganizationPublicKeyCache>,
    State(db): State<Database>,
    State(s3_client): State<S3Client>,
    Json(form): Json<RetrieveUploadUrlForm>,
) -> Result<String, AppError> {
    let (keys, _max_epoch) = db
        .hierarchy_queries
        .key_hierarchy(&anchor_org_pks.get().await, time::now())
        .await?;

    let (signing_journalist_id, signing_journalist_id_pk) = keys
        .find_journalist_id_pk_from_raw_ed25519_pk(form.signing_pk())
        .ok_or(AppError::SigningKeyNotFound)?;

    // Check the form is valid
    form.to_verified_form_data(signing_journalist_id_pk, time::now())
        .map_err(|e| {
            tracing::error!("Failed to verify form {:?}", e);
            AppError::SignatureVerificationFailed
        })?;

    let stage = get_env_or_error("STAGE")?;
    let stage = Stage::from_str(&stage, true).map_err(|e| {
        tracing::error!("Failed to convert STAGE to enum {:?}", &e);
        AppError::IncorrectStageFound(e)
    })?;

    let url_expiry_in_seconds = HOUR_IN_SECONDS as u64;

    let created_at = time::now().to_rfc3339();

    // Full s3://journalist-vault-backups-prod/journalist_id_1/2025-11-20T16:00:24.381734+00:00.backup
    let backup_bucket_name = get_backup_bucket_name(&stage);

    let filepath = format!("{}/{}.backup", signing_journalist_id, created_at);

    if let Ok(presigned_url) = s3_client
        .create_presigned_put_object_url(&backup_bucket_name, &filepath, url_expiry_in_seconds)
        .await
    {
        Ok(presigned_url)
    } else {
        Err(AppError::S3PresignedUrlError)
    }
}

pub async fn post_backup_signing_pk(
    State(anchor_org_pks): State<AnchorOrganizationPublicKeyCache>,
    State(db): State<Database>,
    Json(body): Json<PostBackupIdKeyForm>,
) -> Result<(), AppError> {
    let now = time::now();
    let (keys, _max_epoch) = db
        .hierarchy_queries
        .key_hierarchy(&anchor_org_pks.get().await, now)
        .await?;

    let org_signing_key = keys
        .find_org_pk_from_raw_ed25519_pk(body.signing_pk())
        .ok_or(AppError::SigningKeyNotFound)?;

    let Ok(backup_signing_key) = body.to_verified_form_data(org_signing_key, now) else {
        return Err(AppError::SignatureVerificationFailed);
    };

    let verified_backup_key = verify_backup_id_pk(&backup_signing_key, org_signing_key, now)?;

    db.backup_key_queries
        .insert_backup_id_pk(&verified_backup_key, org_signing_key)
        .await?;
    Ok(())
}

pub async fn post_backup_encryption_pk(
    State(db): State<Database>,
    Json(body): Json<PostBackupMsgKeyForm>,
) -> Result<(), AppError> {
    let now = time::now();
    let backup_signing_key = db
        .backup_key_queries
        .find_backup_signing_pk_from_ed25519_pk(body.signing_pk(), now)
        .await?
        .ok_or(AppError::SigningKeyNotFound)?;

    let Ok(backup_encryption_key) = body.to_verified_form_data(&backup_signing_key, now) else {
        return Err(AppError::SignatureVerificationFailed);
    };

    let verified_backup_key =
        verify_backup_msg_pk(&backup_encryption_key, &backup_signing_key, now)?;

    db.backup_key_queries
        .insert_backup_encryption_pk(&verified_backup_key, &backup_signing_key)
        .await?;

    Ok(())
}
