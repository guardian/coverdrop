use axum::{Extension, Json};
use common::{
    protocol::keys::{
        LatestKey, UntrustedAnchorOrganizationPublicKey, UntrustedCoverNodeProvisioningPublicKey,
        UntrustedJournalistProvisioningPublicKey,
    },
    time,
};
use identity_api_database::Database;
use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Serialize, Deserialize)]
pub struct IdentityApiPublicKeys {
    anchor_org_pks: Vec<UntrustedAnchorOrganizationPublicKey>,
    covernode_provisioning_pk: Option<UntrustedCoverNodeProvisioningPublicKey>,
    journalist_provisioning_pk: Option<UntrustedJournalistProvisioningPublicKey>,
}

pub async fn get_public_keys(
    Extension(database): Extension<Database>,
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

    Ok(Json(IdentityApiPublicKeys {
        anchor_org_pks,
        covernode_provisioning_pk,
        journalist_provisioning_pk,
    }))
}
