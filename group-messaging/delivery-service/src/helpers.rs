use common::api::api_client::ApiClient;
use common::api::models::journalist_id::JournalistIdentity;
use common::crypto::keys::Ed25519PublicKey;
use common::protocol::keys::{AnchorOrganizationPublicKey, JournalistIdPublicKey};
use common::time;

use crate::error::DeliveryServiceError;

/// Fetches and verifies public keys from the identity API, then finds the journalist
/// identity and verifying key for the given signing public key.
///
/// This is a common pattern used across all delivery service endpoints to authenticate
/// the request by verifying the signing key against the trusted key hierarchy.
///
/// # Returns
/// A tuple of (JournalistIdentity, JournalistIdPublicKey) if the key is found and verified,
/// or an appropriate error if the key fetch fails or the signing key is not found.
pub async fn fetch_and_verify_journalist_key(
    api_client: &ApiClient,
    trust_anchors: &[AnchorOrganizationPublicKey],
    signing_pk: &Ed25519PublicKey,
) -> Result<(JournalistIdentity, JournalistIdPublicKey), DeliveryServiceError> {
    // Fetch and verify public keys
    let verified_public_keys_and_profiles = api_client
        .get_public_keys()
        .await
        .map_err(|e| {
            DeliveryServiceError::Anyhow(anyhow::anyhow!("Failed to fetch public keys: {}", e))
        })?
        .into_trusted(trust_anchors, time::now());

    let (client_id, verifying_id_pk) = verified_public_keys_and_profiles
        .keys
        .find_journalist_id_pk_from_raw_ed25519_pk(signing_pk)
        .ok_or(DeliveryServiceError::SigningKeyNotFound)?;

    Ok((client_id.clone(), verifying_id_pk.clone()))
}
