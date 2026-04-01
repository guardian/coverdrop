use axum::extract::State;
use axum::Json;
use common::api::api_client::ApiClient;
use common::api::models::journalist_id::JournalistIdentity;
use common::protocol::keys::AnchorOrganizationPublicKey;
use common::time;
use openmls::prelude::KeyPackageIn;
use std::sync::Arc;

use crate::error::DeliveryServiceError;
use crate::helpers::fetch_and_verify_journalist_key;
use crate::services::database::Database;
use delivery_service_lib::forms::{
    ConsumeKeyPackageForm, GetClientsForm, PublishKeyPackagesForm, RegisterClientForm,
};

/// Register a new client with their key packages.
/// If a client with this ID already exists, returns 200 OK.
pub async fn register_client(
    State(db): State<Database>,
    State(api_client): State<ApiClient>,
    State(trust_anchors): State<Arc<Vec<AnchorOrganizationPublicKey>>>,
    Json(form): Json<RegisterClientForm>,
) -> Result<(), DeliveryServiceError> {
    let (client_id, verifying_id_pk) =
        fetch_and_verify_journalist_key(&api_client, &trust_anchors, form.signing_pk()).await?;

    let body = form
        .to_verified_form_data(&verifying_id_pk, time::now())
        .map_err(|e| {
            tracing::error!("Failed to verify form {}", e);
            DeliveryServiceError::SignatureVerificationFailed
        })?;

    // Validate that we have at least one key package
    if body.key_packages.is_empty() {
        tracing::error!("Register client request has no key packages");
        return Err(DeliveryServiceError::MissingKeyPackages);
    }

    tracing::debug!("Registering client: {}", client_id);

    // Check if client already exists
    if db.client_queries.client_exists(&client_id).await? {
        tracing::debug!("Client {} already exists, returning OK", client_id);
        return Ok(());
    }

    // Register the client with key packages in a single transaction
    db.client_queries
        .register_client(&client_id, body.key_packages)
        .await?;

    tracing::info!("Successfully registered client: {}", client_id);

    Ok(())
}

pub async fn get_clients(
    State(db): State<Database>,
    State(api_client): State<ApiClient>,
    State(trust_anchors): State<Arc<Vec<AnchorOrganizationPublicKey>>>,
    Json(form): Json<GetClientsForm>,
) -> Result<Json<Vec<JournalistIdentity>>, DeliveryServiceError> {
    let (_client_id, verifying_id_pk) =
        fetch_and_verify_journalist_key(&api_client, &trust_anchors, form.signing_pk()).await?;

    // Verify the form
    let _body = form
        .to_verified_form_data(&verifying_id_pk, time::now())
        .map_err(|e| {
            tracing::error!("Failed to verify form {}", e);
            DeliveryServiceError::SignatureVerificationFailed
        })?;

    let client_ids = db.client_queries.get_all_client_ids().await?;
    Ok(Json(client_ids))
}

/// Publish additional key packages for an existing client.
///
/// The client ID is derived from the form signing key.
pub async fn publish_key_packages(
    State(db): State<Database>,
    State(api_client): State<ApiClient>,
    State(trust_anchors): State<Arc<Vec<AnchorOrganizationPublicKey>>>,
    Json(form): Json<PublishKeyPackagesForm>,
) -> Result<(), DeliveryServiceError> {
    let (client_id, verifying_id_pk) =
        fetch_and_verify_journalist_key(&api_client, &trust_anchors, form.signing_pk()).await?;

    let body = form
        .to_verified_form_data(&verifying_id_pk, time::now())
        .map_err(|e| {
            tracing::error!("Failed to verify form {}", e);
            DeliveryServiceError::SignatureVerificationFailed
        })?;

    if body.key_packages.is_empty() {
        tracing::error!("Publish key packages request has no key packages");
        return Err(DeliveryServiceError::MissingKeyPackages);
    }

    if !db.client_queries.client_exists(&client_id).await? {
        return Err(DeliveryServiceError::ClientNotFound(format!(
            "Client {} not found",
            client_id
        )));
    }

    tracing::debug!(
        "Publishing {} key packages for client: {}",
        body.key_packages.len(),
        client_id
    );

    db.client_queries
        .insert_key_packages(&client_id, body.key_packages)
        .await?;

    tracing::info!(
        "Successfully published key packages for client: {}",
        client_id
    );

    Ok(())
}

pub async fn consume_key_package(
    State(db): State<Database>,
    State(api_client): State<ApiClient>,
    State(trust_anchors): State<Arc<Vec<AnchorOrganizationPublicKey>>>,
    Json(form): Json<ConsumeKeyPackageForm>,
) -> Result<Json<KeyPackageIn>, DeliveryServiceError> {
    let (_client_id, verifying_id_pk) =
        fetch_and_verify_journalist_key(&api_client, &trust_anchors, form.signing_pk()).await?;

    let body = form
        .to_verified_form_data(&verifying_id_pk, time::now())
        .map_err(|e| {
            tracing::error!("Failed to verify form {}", e);
            DeliveryServiceError::SignatureVerificationFailed
        })?;

    let journalist_id = body.target_client_id;

    if !db.client_queries.client_exists(&journalist_id).await? {
        return Err(DeliveryServiceError::ClientNotFound(format!(
            "Client {} not found",
            journalist_id
        )));
    }

    let key_package = db
        .client_queries
        .consume_key_package(&journalist_id)
        .await?;

    match key_package {
        Some(kp) => Ok(Json(kp)),
        None => Err(DeliveryServiceError::KeyPackagesDepleted(format!(
            "No key packages available for client {}",
            journalist_id
        ))),
    }
}
