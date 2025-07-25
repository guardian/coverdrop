use axum::{extract::State, Json};
use common::{api::forms::PatchJournalistStatusForm, time};

use crate::{
    anchor_org_pk_cache::AnchorOrganizationPublicKeyCache, error::AppError,
    services::database::Database,
};

pub async fn patch_journalist_status(
    State(anchor_org_pks): State<AnchorOrganizationPublicKeyCache>,
    State(db): State<Database>,
    Json(form): Json<PatchJournalistStatusForm>,
) -> Result<(), AppError> {
    let (keys, _max_epoch) = db
        .hierarchy_queries
        .key_hierarchy(&anchor_org_pks.get().await, time::now())
        .await?;

    let (signing_journalist_id, signing_journalist_id_pk) = keys
        .find_journalist_id_pk_from_raw_ed25519_pk(form.signing_pk())
        .ok_or(AppError::SigningKeyNotFound)?;

    // Check the form is valid
    let form_body = form
        .to_verified_form_data(signing_journalist_id_pk, time::now())
        .map_err(|e| {
            tracing::error!("Failed to verify form {:?}", e);
            AppError::SignatureVerificationFailed
        })?;

    // make sure that the journalist id in the form is the same as the one which signed the form
    if &form_body.journalist_id != signing_journalist_id {
        tracing::error!(
            "Journalist {} not authorized to update journalist {}",
            signing_journalist_id,
            &form_body.journalist_id
        );
        return Err(AppError::JournalistUnauthorized);
    }

    db.journalist_queries
        .update_journalist_status(form_body.journalist_id, form_body.status)
        .await?;

    Ok(())
}
