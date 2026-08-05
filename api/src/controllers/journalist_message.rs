use axum::extract::State;
use axum::Json;
use common::api::forms::PostJournalistToCoverNodeMessageForm;
#[allow(deprecated)]
use common::api::forms::PostJournalistToCoverNodeMessageFormDeprecated;
use common::{aws::kinesis::client::KinesisClient, time};

use crate::{
    anchor_org_pk_cache::AnchorOrganizationPublicKeyCache, error::AppError,
    services::database::Database,
};

/// Forwards a journalist to CoverNode message to the appropriate kinesis
/// stream. The message is wrapped in a `PostJournalistToCoverNodeMessageForm`
/// which allows us to verify the message is from a real and trusted journalist.
/// It does not reveal any information about if the message is cover or real.
#[deprecated(
    note = "Use `post_forward_journalist_to_covernode_msg_with_deduplication_token` instead"
)]
#[allow(deprecated)]
pub async fn post_forward_journalist_to_covernode_msg(
    State(anchor_org_pks): State<AnchorOrganizationPublicKeyCache>,
    State(db): State<Database>,
    State(kinesis_client): State<KinesisClient>,
    Json(form): Json<PostJournalistToCoverNodeMessageFormDeprecated>,
) -> Result<(), AppError> {
    let (keys, _max_epoch) = db
        .hierarchy_queries
        .key_hierarchy(&anchor_org_pks.get().await, time::now())
        .await?;

    let (_, journalist_id_pk) = keys
        .find_journalist_id_pk_from_raw_ed25519_pk(form.signing_pk())
        .ok_or(AppError::SigningKeyNotFound)?;

    // Check the form is valid
    let j2c_msg = form
        .to_verified_form_data(journalist_id_pk, time::now())
        .map_err(|e| {
            tracing::error!("Failed to verify form {:?}", e);
            AppError::SignatureVerificationFailed
        })?;

    kinesis_client
        .encode_and_put_journalist_message(j2c_msg)
        .await
        .map_err(|e| {
            tracing::error!("Failed to put J2C message on kinesis: {:?}", e);
            AppError::KinesisStreamPut
        })?;

    Ok(())
}

/// Forwards a journalist to CoverNode message to the appropriate kinesis
/// stream if its deduplication hasn't been seen before, otherwise it is ignored.
pub async fn post_forward_journalist_to_covernode_msg_with_deduplication_token(
    State(anchor_org_pks): State<AnchorOrganizationPublicKeyCache>,
    State(db): State<Database>,
    State(kinesis_client): State<KinesisClient>,
    Json(form): Json<PostJournalistToCoverNodeMessageForm>,
) -> Result<(), AppError> {
    let (keys, _max_epoch) = db
        .hierarchy_queries
        .key_hierarchy(&anchor_org_pks.get().await, time::now())
        .await?;

    let (_, journalist_id_pk) = keys
        .find_journalist_id_pk_from_raw_ed25519_pk(form.signing_pk())
        .ok_or(AppError::SigningKeyNotFound)?;

    // Check the form is valid
    let j2c_msg_body = form
        .to_verified_form_data(journalist_id_pk, time::now())
        .map_err(|e| {
            tracing::error!("Failed to verify form {:?}", e);
            AppError::SignatureVerificationFailed
        })?;

    let duplicate = !db
        .j2c_deduplication_id_queries
        .insert_deduplication_id(j2c_msg_body.deduplication_id)
        .await?;

    if duplicate {
        tracing::info!(
            "Duplicate J2C message with deduplication_id {} ignored",
            j2c_msg_body.deduplication_id
        );
        return Ok(());
    }

    kinesis_client
        .encode_and_put_journalist_message(j2c_msg_body.message)
        .await
        .map_err(|e| {
            tracing::error!("Failed to put J2C message on kinesis: {:?}", e);
            AppError::KinesisStreamPut
        })?;

    Ok(())
}
