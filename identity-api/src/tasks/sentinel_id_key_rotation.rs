use async_trait::async_trait;
use common::{
    api::{
        api_client::ApiClient,
        forms::PostSentinelIdPublicKeyForm,
        models::{
            sentinel_id::SentinelIdentity,
            sentinel_id_and_id_pk_rotation_form::SentinelIdAndPublicKeyRotationForm,
        },
    },
    crypto::keys::{public_key::PublicKey as _, Ed25519PublicKey},
    protocol::keys::{
        sign_sentinel_id_pk, JournalistProvisioningKeyPair, OrganizationPublicKeyFamilyList,
        UnregisteredSentinelIdPublicKey,
    },
};

use super::id_key_to_rotate::IdKeyToRotate;

/// Marker type for sentinel ID public key rotation.
pub struct SentinelIdKeyRotation;

#[async_trait]
impl IdKeyToRotate for SentinelIdKeyRotation {
    type Identity = SentinelIdentity;
    type IdAndPKRotationForm = SentinelIdAndPublicKeyRotationForm;

    fn task_name() -> &'static str {
        "rotate_sentinel_id_public_keys"
    }

    async fn get_rotation_forms(
        api_client: &ApiClient,
    ) -> anyhow::Result<Vec<Self::IdAndPKRotationForm>> {
        api_client.get_sentinel_id_pk_forms().await
    }

    fn form_identity(entry: &Self::IdAndPKRotationForm) -> &Self::Identity {
        &entry.sentinel_id
    }

    fn verify_and_extract(
        id_and_pk_rotation_form: &Self::IdAndPKRotationForm,
        keys: &OrganizationPublicKeyFamilyList,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Option<(Self::Identity, Ed25519PublicKey)> {
        let form = &id_and_pk_rotation_form.form;

        let Some((sentinel_id, verifying_pk)) =
            keys.find_sentinel_id_pk_from_raw_ed25519_pk(form.signing_pk())
        else {
            tracing::error!("No sentinel ID and verifying pk found for form signing pk");
            return None;
        };

        let Ok(verified_form) = form.to_verified_form_data(verifying_pk, now) else {
            tracing::error!(
                "Could not verify form data for {}'s key rotation form",
                sentinel_id
            );
            return None;
        };

        let new_pk = verified_form.new_pk.to_trusted();
        Some((sentinel_id.clone(), new_pk.key))
    }

    fn check_for_existing_key(
        keys: &OrganizationPublicKeyFamilyList,
        identity: &Self::Identity,
        new_pk_raw: &Ed25519PublicKey,
    ) -> anyhow::Result<()> {
        if let Some((existing_sentinel_id, existing_sentinel_id_pk)) =
            keys.find_sentinel_id_pk_from_raw_ed25519_pk(new_pk_raw)
        {
            tracing::warn!(
                "Sentinel ID {} has attempted to upload a key that already exists: {}",
                existing_sentinel_id,
                existing_sentinel_id_pk.public_key_hex()
            );

            // This key already exists but is registered to a different sentinel ID.
            // This should not happen.
            if existing_sentinel_id != identity {
                anyhow::bail!(
                    "Key exists already but has been registered to a different sentinel ID"
                );
            }
        }
        Ok(())
    }

    async fn sign_and_post(
        api_client: &ApiClient,
        identity: Self::Identity,
        new_pk_raw: Ed25519PublicKey,
        provisioning_key_pair: &JournalistProvisioningKeyPair,
        now: chrono::DateTime<chrono::Utc>,
    ) -> anyhow::Result<()> {
        let new_pk = UnregisteredSentinelIdPublicKey::new(new_pk_raw);

        let signed_sentinel_id_pk = sign_sentinel_id_pk(new_pk, provisioning_key_pair, now);

        tracing::debug!(
            "Signed new sentinel id public key for {}: {}",
            identity,
            &serde_json::to_string(&signed_sentinel_id_pk.to_untrusted())
                .unwrap_or_else(|e| format!("<failed to serialize: {e}>"))
        );

        let form = PostSentinelIdPublicKeyForm::new(
            identity,
            signed_sentinel_id_pk.to_untrusted(),
            true,
            provisioning_key_pair,
            now,
        )?;

        api_client.post_sentinel_id_pk_form(form).await?;
        Ok(())
    }
}
