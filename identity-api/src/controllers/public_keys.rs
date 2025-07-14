use axum::{extract::State, Json};
use common::{identity_api::models::IdentityApiPublicKeys, protocol::keys::LatestKey, time};
use identity_api_database::Database;

use crate::error::AppError;

pub async fn get_public_keys(
    State(database): State<Database>,
) -> Result<Json<IdentityApiPublicKeys>, AppError> {
    let now = time::now();

    let anchor_org_pks = database
        .select_anchor_organization_pks(now)
        .await
        .map_err(AppError::DatabaseError)?
        .into_iter()
        .map(|pk| pk.to_untrusted())
        .collect();

    let covernode_provisioning_pk = database
        .select_covernode_provisioning_key_pairs(now)
        .await
        .map_err(AppError::DatabaseError)?
        .into_latest_key()
        .map(|key_pair| key_pair.public_key().to_untrusted());

    let journalist_provisioning_pk = database
        .select_journalist_provisioning_key_pairs(now)
        .await
        .map_err(AppError::DatabaseError)?
        .into_latest_key()
        .map(|key_pair| key_pair.public_key().to_untrusted());

    Ok(Json(IdentityApiPublicKeys::new(
        anchor_org_pks,
        covernode_provisioning_pk,
        journalist_provisioning_pk,
    )))
}
