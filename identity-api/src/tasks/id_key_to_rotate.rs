use async_trait::async_trait;
use chrono::DateTime;
use common::{
    api::api_client::ApiClient,
    crypto::keys::Ed25519PublicKey,
    protocol::keys::{JournalistProvisioningKeyPair, OrganizationPublicKeyFamilyList},
};

/// Trait that abstracts rotating journalist ID  and sentinel ID public keys.
#[async_trait]
pub trait IdKeyToRotate: Send + Sync {
    /// The identity type (JournalistIdentity or SentinelIdentity)
    type Identity: std::fmt::Display + Clone + PartialEq + Send + Sync;

    /// The rotation form entry type returned from the API
    type IdAndPKRotationForm: Send + Sync;

    fn task_name() -> &'static str;

    /// Fetch queued rotation forms from the API
    async fn get_rotation_forms(
        api_client: &ApiClient,
    ) -> anyhow::Result<Vec<Self::IdAndPKRotationForm>>;

    /// Extract the identity from a rotation form entry (from the form metadata).
    fn form_identity(entry: &Self::IdAndPKRotationForm) -> &Self::Identity;

    /// Look up the identity and verifying public key from the key list using the
    /// form's signing public key. Returns None if no match is found.
    ///
    /// Also verifies the form signature and returns the new public key's raw bytes
    /// if verification succeeds.
    ///
    /// Returns: Ok(Some((identity, new_pk_raw))) on success
    ///          Ok(None) if the form should be skipped (logged internally)
    fn verify_and_extract(
        id_and_pk_rotation_form: &Self::IdAndPKRotationForm,
        keys: &OrganizationPublicKeyFamilyList,
        now: DateTime<chrono::Utc>,
    ) -> Option<(Self::Identity, Ed25519PublicKey)>;

    /// Check if the new key already exists in the key list.
    /// Returns Ok(()) if the key doesn't exist or exists for the same identity.
    /// Returns Err if the key exists but belongs to a different identity.
    fn check_for_existing_key(
        keys: &OrganizationPublicKeyFamilyList,
        identity: &Self::Identity,
        new_pk_raw: &Ed25519PublicKey,
    ) -> anyhow::Result<()>;

    /// Sign the new public key and post it to the API
    async fn sign_and_post(
        api_client: &ApiClient,
        identity: Self::Identity,
        new_pk_raw: Ed25519PublicKey,
        provisioning_key_pair: &JournalistProvisioningKeyPair,
        now: DateTime<chrono::Utc>,
    ) -> anyhow::Result<()>;
}
