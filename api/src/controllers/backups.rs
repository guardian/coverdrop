use crate::anchor_org_pk_cache::AnchorOrganizationPublicKeyCache;
use crate::error::AppError;
use crate::services::database::Database;
use axum::extract::State;
use axum::Json;
use common::api::forms::{
    GetBackupDataForm, PostBackupDataForm, PostBackupIdKeyForm, PostBackupMsgKeyForm,
};
use common::backup::keys::{verify_backup_id_pk, verify_backup_msg_pk, BackupIdPublicKey};
use common::protocol::backup_data::BackupDataWithSignature;
use common::time;
use http::HeaderMap;

pub const BACKUP_DATA_MAX_SIZE_BYTES: usize = 300 * 1024 * 1024;

pub async fn get_backup_data(
    State(anchor_org_pks): State<AnchorOrganizationPublicKeyCache>,
    State(db): State<Database>,
    Json(body): Json<GetBackupDataForm>,
) -> Result<(HeaderMap, Json<BackupDataWithSignature>), AppError> {
    let now = time::now();
    let (keys, _max_epoch) = db
        .hierarchy_queries
        .key_hierarchy(&anchor_org_pks.get().await, now)
        .await?;

    let backup_pk: BackupIdPublicKey = db
        .backup_key_queries
        .find_backup_signing_pk_from_ed25519_pk(body.signing_pk(), now)
        .await?
        .ok_or(AppError::SigningKeyNotFound)?;

    let Ok(journalist_identity) = body.to_verified_form_data(&backup_pk, now) else {
        return Err(AppError::SignatureVerificationFailed);
    };

    let backup_data = db
        .backup_data_queries
        .get_latest_backup_data(keys, &journalist_identity)
        .await?;

    let headers = HeaderMap::new();
    Ok((headers, Json(backup_data.to_unverified()?)))
}

pub async fn post_backup_data(
    State(anchor_org_pks): State<AnchorOrganizationPublicKeyCache>,
    State(db): State<Database>,
    Json(body): Json<PostBackupDataForm>,
) -> Result<(), AppError> {
    let now = time::now();
    let (keys, _max_epoch) = db
        .hierarchy_queries
        .key_hierarchy(&anchor_org_pks.get().await, now)
        .await?;

    let (signing_journalist_id, signing_journalist_id_pk) = keys
        .find_journalist_id_pk_from_raw_ed25519_pk(body.signing_pk())
        .ok_or(AppError::SigningKeyNotFound)?;

    let Ok(backup_data) = body.to_verified_form_data(signing_journalist_id_pk, now) else {
        return Err(AppError::SignatureVerificationFailed);
    };

    // Check the journalist_id of the backup's signing key matches the identity associated with the
    // signing key from the form. The signing keys might be different.
    if let Some((signing_identity_from_backup_data, _)) =
        keys.find_journalist_id_pk_from_raw_ed25519_pk(&backup_data.signed_with().key)
    {
        if signing_identity_from_backup_data != signing_journalist_id {
            return Err(AppError::JournalistIdDoesNotMatchSigningKey(
                signing_identity_from_backup_data.clone(),
            ));
        }
    } else {
        return Err(AppError::SigningKeyNotFound);
    }

    let verified_backup_data = backup_data.to_verified(signing_journalist_id_pk, now)?;

    // Check the identity in the backup data matches the identity associated with the signing key
    if verified_backup_data.backup_data()?.journalist_identity != *signing_journalist_id {
        return Err(AppError::JournalistIdDoesNotMatchSigningKey(
            signing_journalist_id.clone(),
        ));
    }

    db.backup_data_queries
        .store_backup_data(&verified_backup_data)
        .await?;

    Ok(())
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
