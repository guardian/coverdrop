use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use common::api::api_client::ApiClient;
use common::protocol::keys::AnchorOrganizationPublicKey;
use common::time;
use delivery_service_lib::models::GroupMessage;
use openmls::prelude::{MlsMessageBodyIn, MlsMessageIn, WireFormat};
use std::sync::Arc;

use crate::error::DeliveryServiceError;
use crate::helpers::fetch_and_verify_journalist_key;
use crate::services::database::Database;
use delivery_service_lib::forms::{AddMembersForm, ReceiveMessagesForm, SendMessageForm};

/// Add members to a group.
///
/// This endpoint receives a JSON payload with TLS-serialized messages:
/// - A Welcome message for new members being added to the group
/// - A Commit message for existing members to update their group state
///
/// Both messages are stored atomically to ensure consistent group membership state.
pub async fn add_members(
    State(db): State<Database>,
    State(api_client): State<ApiClient>,
    State(trust_anchors): State<Arc<Vec<AnchorOrganizationPublicKey>>>,
    Json(form): Json<AddMembersForm>,
) -> Result<StatusCode, DeliveryServiceError> {
    let (_client_id, verifying_id_pk) =
        fetch_and_verify_journalist_key(&api_client, &trust_anchors, form.signing_pk()).await?;

    let body = form
        .to_verified_form_data(&verifying_id_pk, time::now())
        .map_err(|e| {
            tracing::error!("Failed to verify form {}", e);
            DeliveryServiceError::SignatureVerificationFailed
        })?;

    // Deserialize the welcome message
    let welcome_msg = body
        .welcome_message
        .deserialize::<MlsMessageIn>()
        .map_err(|e| {
            DeliveryServiceError::DeserializationError(format!(
                "Failed to deserialize welcome: {}",
                e
            ))
        })?;

    // Deserialize the commit message
    let commit_msg = body
        .commit_message
        .deserialize::<MlsMessageIn>()
        .map_err(|e| {
            DeliveryServiceError::DeserializationError(format!(
                "Failed to deserialize commit: {}",
                e
            ))
        })?;

    // Verify the welcome is actually a Welcome message
    // check the format of the welcome and commit messages before storing
    let welcome = match welcome_msg.extract() {
        MlsMessageBodyIn::Welcome(welcome) => welcome,
        _ => {
            return Err(DeliveryServiceError::MalformedMlsMessage(
                "Failed to extract Welcome from message".to_string(),
            ))
        }
    };

    // Check the commit message is a private message
    if commit_msg.wire_format() != WireFormat::PrivateMessage {
        return Err(DeliveryServiceError::MalformedMlsMessage(
            "Expected add member commit to be a private message".to_string(),
        ));
    };

    tracing::debug!(
        "Storing welcome message with {} secrets and commit for {} existing members",
        welcome.secrets().len(),
        body.existing_members.len()
    );

    // Store both messages atomically
    db.message_queries
        .store_add_members_messages(
            commit_msg,
            &body.existing_members,
            &body.new_members,
            time::now(),
            &body.welcome_message,
            &body.commit_message,
        )
        .await?;

    Ok(StatusCode::OK)
}

/// Send a message to a group.
///
/// This endpoint receives a JSON payload with a TLS-serialized
/// GroupMessage containing an MLS message and a list of recipient client IDs.
/// For handshake messages, it validates that the epoch is not stale before storing.
pub async fn send_message(
    State(db): State<Database>,
    State(api_client): State<ApiClient>,
    State(trust_anchors): State<Arc<Vec<AnchorOrganizationPublicKey>>>,
    Json(form): Json<SendMessageForm>,
) -> Result<StatusCode, DeliveryServiceError> {
    let (_client_id, verifying_id_pk) =
        fetch_and_verify_journalist_key(&api_client, &trust_anchors, form.signing_pk()).await?;

    let body = form
        .to_verified_form_data(&verifying_id_pk, time::now())
        .map_err(|e| {
            tracing::error!("Failed to verify form {}", e);
            DeliveryServiceError::SignatureVerificationFailed
        })?;

    // Deserialize the MLS message
    let mls_message_in = body.message.deserialize::<MlsMessageIn>().map_err(|e| {
        DeliveryServiceError::DeserializationError(format!(
            "Failed to deserialize group message: {}",
            e
        ))
    })?;

    tracing::debug!(
        "Received group message for {} recipients",
        body.recipients.len()
    );

    // Store the message (this also handles epoch validation for handshake messages)
    db.message_queries
        .store_group_message(mls_message_in, &body.recipients, time::now(), &body.message)
        .await?;

    Ok(StatusCode::OK)
}

/// Receive messages for a client.
///
/// This endpoint receives a JSON payload with ids_greater_than parameter.
/// It returns all welcome messages and regular messages for that client since the specified ID,
/// in the order they arrived.
pub async fn receive_messages(
    State(db): State<Database>,
    State(api_client): State<ApiClient>,
    State(trust_anchors): State<Arc<Vec<AnchorOrganizationPublicKey>>>,
    Json(form): Json<ReceiveMessagesForm>,
) -> Result<Json<Vec<GroupMessage>>, DeliveryServiceError> {
    let (client_id, verifying_id_pk) =
        fetch_and_verify_journalist_key(&api_client, &trust_anchors, form.signing_pk()).await?;

    let body = form
        .to_verified_form_data(&verifying_id_pk, time::now())
        .map_err(|e| {
            tracing::error!("Failed to verify form {}", e);
            DeliveryServiceError::SignatureVerificationFailed
        })?;

    tracing::debug!(
        "Receiving messages for client {} with IDs greater than {}",
        client_id,
        body.ids_greater_than
    );

    // Check if client exists
    if !db.client_queries.client_exists(&client_id).await? {
        return Err(DeliveryServiceError::ClientNotFound(format!(
            "Client {} not found",
            client_id
        )));
    }

    // Fetch all messages with IDs greater than the specified value
    let messages = db
        .message_queries
        .get_messages_since(&client_id, body.ids_greater_than)
        .await?;

    tracing::info!(
        "Returning {} messages for client {}",
        messages.len(),
        client_id
    );

    Ok(Json(messages))
}
