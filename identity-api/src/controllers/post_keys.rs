use crate::error::AppError;
use axum::{http::StatusCode, Extension, Json};
use common::{
    api::api_client::ApiClient,
    crypto::keys::public_key::PublicKey,
    identity_api::{
        forms::post_rotate_covernode_id::RotateCoverNodeIdPublicKeyForm,
        models::UntrustedCoverNodeIdPublicKeyWithEpoch,
    },
    protocol::keys::{sign_covernode_id_pk, LatestKey as _},
    time,
};
use identity_api_database::Database;

pub async fn post_rotate_covernode_id_key(
    Extension(api_client): Extension<ApiClient>,
    Extension(database): Extension<Database>,
    Json(form): Json<RotateCoverNodeIdPublicKeyForm>,
) -> Result<(StatusCode, Json<UntrustedCoverNodeIdPublicKeyWithEpoch>), AppError> {
    let anchor_org_pks = database
        .select_anchor_organization_pks(time::now())
        .await
        .map_err(AppError::DatabaseError)?;

    let keys = api_client.get_public_keys().await.map(|keys_and_profiles| {
        keys_and_profiles
            .into_trusted(&anchor_org_pks, time::now())
            .keys
    });

    let keys = match keys {
        Ok(keys) => keys,
        Err(e) => return Err(AppError::from_api_client_error(e)),
    };

    // Which CoverNode is rotating their key?
    //
    // The path for this endpoint uses the "me" keyword, which is a special identifier
    // for the CoverNode sending the request.
    //
    // This allows us to side-step a potential Insecure Direct Object Reference vulnerability:
    // https://en.wikipedia.org/wiki/Insecure_direct_object_reference
    //
    // If the CoverNode exists, but someone is maliciously trying to rotate their key
    // the form signature will not be valid, unless they also have that CoverNode's
    // secret key - at which point all bets are off.
    let Some((covernode_id, verifying_pk)) =
        keys.find_covernode_id_pk_from_raw_ed25519_pk(form.signing_pk())
    else {
        return Err(AppError::UnknownCoverNodeIdentityOrCoverNodeIdKey);
    };

    //
    // Verification of the form
    //

    // Is the form signature valid?
    let Ok(verified_form) = form.to_verified_form_data(verifying_pk, time::now()) else {
        return Err(AppError::CertificateDataVerificationFailed);
    };

    let new_pk = verified_form.new_pk.to_trusted();

    // Has this key already been uploaded?
    if let Some((existing_covernode_id, existing_covernode_id_pk)) =
        keys.find_covernode_id_pk_from_raw_ed25519_pk(&new_pk.key)
    {
        tracing::warn!(
            "CoverNode {} has attempted to upload a key that already exists: {}",
            existing_covernode_id,
            existing_covernode_id_pk.public_key_hex()
        );

        // This key already exists but is registered to a different CoverNode.
        // This should not happen.
        if existing_covernode_id != covernode_id {
            return Err(AppError::CoverNodeIdRotationIdentityMismatch);
        }

        // The new key already exists in the API but the identity API does not know it's epoch
        // So we need to request the key and epoch from the API anyway
    }

    //
    // Everything is ok with our form! Let's rotate the key
    //

    let covernode_provisioning_key_pair = database
        .select_covernode_provisioning_key_pairs(time::now())
        .await
        .map_err(AppError::DatabaseError)?
        .into_latest_key_required()?;

    let signed_covernode_id_pk =
        sign_covernode_id_pk(new_pk, &covernode_provisioning_key_pair, time::now());

    tracing::debug!(
        "Signed new CoverNode id public key: {}",
        &serde_json::to_string(&signed_covernode_id_pk.to_untrusted())
            .unwrap_or_else(|e| format!("<failed to serialize: {}>", e))
    );

    let api_response = api_client
        .post_covernode_id_pk(
            covernode_id,
            &signed_covernode_id_pk,
            &covernode_provisioning_key_pair,
            time::now(),
        )
        .await;

    match api_response {
        Ok(epoch) => Ok((
            StatusCode::CREATED,
            Json(UntrustedCoverNodeIdPublicKeyWithEpoch {
                key: signed_covernode_id_pk.to_untrusted(),
                epoch,
            }),
        )),
        Err(e) => Err(AppError::from_api_client_error(e)),
    }
}
