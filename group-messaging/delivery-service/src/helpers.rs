use aws_sdk_secretsmanager::Client as SecretsClient;
use common::api::api_client::ApiClient;
use common::api::models::journalist_id::JournalistIdentity;
use common::crypto::keys::Ed25519PublicKey;
use common::protocol::keys::{AnchorOrganizationPublicKey, JournalistIdPublicKey};
use common::time;

use crate::error::DeliveryServiceError;

/// Resolves a database URL by fetching credentials from AWS Secrets Manager.
///
/// The secret is expected to be an RDS-generated secret containing JSON with
/// `username`, `password`, `host`, and `port` fields. The returned URL is in
/// the form `postgresql://username:password@host:port/db_name`.
pub async fn fetch_and_parse_db_url_secret(
    secret_arn: &str,
    db_name: &str,
) -> anyhow::Result<String> {
    let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let client = SecretsClient::new(&config);

    let response = client
        .get_secret_value()
        .secret_id(secret_arn)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch secret {}: {:?}", secret_arn, e))?;

    let secret_string = response
        .secret_string()
        .ok_or_else(|| anyhow::anyhow!("Secret {} has no string value", secret_arn))?;

    let secret: serde_json::Value = serde_json::from_str(secret_string)
        .map_err(|e| anyhow::anyhow!("Failed to parse secret JSON: {}", e))?;

    let username = secret["username"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No 'username' field found in secret"))?;
    let password = secret["password"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No 'password' field found in secret"))?;
    let host = secret["host"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No 'host' field found in secret"))?;
    let port = secret["port"].as_u64().unwrap_or(5432);

    Ok(format!(
        "postgresql://{}:{}@{}:{}/{}",
        username, password, host, port, db_name
    ))
}

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
