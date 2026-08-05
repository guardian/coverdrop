use std::time::Duration;

use anyhow::Result;
use chrono::{DateTime, Utc};
use common::{
    api::{
        api_client::ApiClient,
        forms::{RotateJournalistIdPublicKeyFormForm, RotateSentinelIdPublicKeyFormForm},
    },
    identity_api::forms::{
        post_rotate_journalist_id::RotateJournalistIdPublicKeyForm,
        post_rotate_sentinel_id::RotateSentinelIdPublicKeyForm,
    },
    protocol::{
        constants::{JOURNALIST_ID_KEY_ROTATE_AFTER, SENTINEL_ID_KEY_ROTATE_AFTER},
        keys::{JournalistIdKeyPair, SentinelIdKeyPair},
    },
};
use journalist_vault::{promotable_id_key_pair::PromotableIdKeyPair, JournalistVault};

use crate::constants::{
    JOURNALIST_ID_KEY_POLL_ITERATIONS, JOURNALIST_ID_KEY_POLL_SLEEP_DURATION,
    SENTINEL_ID_KEY_POLL_ITERATIONS, SENTINEL_ID_KEY_POLL_SLEEP_DURATION,
};

/// Trait that extends `PromotableIdKeyPair` with API-level rotation operations.
/// This trait encapsulates the differences between journalist and sentinel
/// key rotation.
pub trait RotatableIdKeyPair: PromotableIdKeyPair {
    /// Check the API for whether a candidate key has already been signed.
    #[allow(async_fn_in_trait)]
    async fn get_pk_with_epoch(
        api: &ApiClient,
        candidate: &Self::Unregistered,
    ) -> Result<Option<Self::SignedWithEpoch>>;

    /// Check whether a valid rotation form already exists in the API queue
    /// for this identity.
    #[allow(async_fn_in_trait)]
    async fn has_valid_queued_form(
        api: &ApiClient,
        vault: &JournalistVault,
        now: DateTime<Utc>,
    ) -> Result<bool>;

    /// Create the inner + outer rotation form and upload it to the API.
    #[allow(async_fn_in_trait)]
    async fn create_and_upload_form(
        api: &ApiClient,
        candidate: &Self::Unregistered,
        latest_key_pair: &Self,
        now: DateTime<Utc>,
    ) -> Result<()>;

    /// Check queued rotation forms and upload a new one if needed.
    /// Default implementation uses `has_valid_queued_form` and `create_and_upload_form`.
    #[allow(async_fn_in_trait)]
    async fn check_and_upload_form_if_needed(
        api: &ApiClient,
        vault: &JournalistVault,
        candidate: &Self::Unregistered,
        latest_key_pair: &Self,
        now: DateTime<Utc>,
    ) -> Result<()> {
        if !Self::has_valid_queued_form(api, vault, now).await? {
            tracing::debug!("Uploading new {} rotation form", Self::key_type_name());
            Self::create_and_upload_form(api, candidate, latest_key_pair, now).await?;
        }
        Ok(())
    }

    fn key_type_name() -> &'static str;
    fn rotate_after() -> chrono::Duration;
    fn poll_iterations() -> u64;
    fn poll_sleep_duration() -> Duration;

    /// Whether rotation should proceed at all.
    /// Returns `Ok(true)` to proceed, `Ok(false)` to skip silently.
    /// The caller already ensures a latest key pair exists; this is for
    /// additional identity-specific checks (e.g. sentinel_id existence).
    #[allow(async_fn_in_trait)]
    async fn should_attempt_rotation(vault: &JournalistVault) -> Result<bool>;
}

impl RotatableIdKeyPair for JournalistIdKeyPair {
    async fn get_pk_with_epoch(
        api: &ApiClient,
        candidate: &Self::Unregistered,
    ) -> Result<Option<Self::SignedWithEpoch>> {
        api.get_journalist_id_pk_with_epoch(candidate.public_key())
            .await
    }

    async fn has_valid_queued_form(
        api: &ApiClient,
        vault: &JournalistVault,
        now: DateTime<Utc>,
    ) -> Result<bool> {
        let journalist_id = vault.journalist_id().await?;
        let current_queued_forms = api.get_journalist_id_pk_forms().await?;
        let maybe_our_form = current_queued_forms
            .iter()
            .find(|f| f.journalist_id == journalist_id);

        Ok(maybe_our_form
            .map(|form| form.form.not_valid_after() >= now)
            .unwrap_or(false))
    }

    async fn create_and_upload_form(
        api: &ApiClient,
        candidate: &Self::Unregistered,
        latest_key_pair: &Self,
        now: DateTime<Utc>,
    ) -> Result<()> {
        let form_for_identity_api =
            RotateJournalistIdPublicKeyForm::new(candidate.public_key(), latest_key_pair, now)?;
        let form_for_api =
            RotateJournalistIdPublicKeyFormForm::new(form_for_identity_api, latest_key_pair, now)?;
        api.post_rotate_journalist_id_pk_form(form_for_api).await?;
        Ok(())
    }

    fn key_type_name() -> &'static str {
        "journalist identity"
    }

    fn rotate_after() -> chrono::Duration {
        JOURNALIST_ID_KEY_ROTATE_AFTER
    }

    fn poll_iterations() -> u64 {
        JOURNALIST_ID_KEY_POLL_ITERATIONS
    }

    fn poll_sleep_duration() -> Duration {
        JOURNALIST_ID_KEY_POLL_SLEEP_DURATION
    }

    async fn should_attempt_rotation(_vault: &JournalistVault) -> Result<bool> {
        Ok(true)
    }
}

impl RotatableIdKeyPair for SentinelIdKeyPair {
    async fn get_pk_with_epoch(
        api: &ApiClient,
        candidate: &Self::Unregistered,
    ) -> Result<Option<Self::SignedWithEpoch>> {
        api.get_sentinel_id_pk_with_epoch(candidate.public_key())
            .await
    }

    async fn has_valid_queued_form(
        api: &ApiClient,
        vault: &JournalistVault,
        now: DateTime<Utc>,
    ) -> Result<bool> {
        let sentinel_id = vault.sentinel_id().await?.ok_or_else(|| {
            anyhow::anyhow!("No sentinel identity set in vault, cannot check rotation forms")
        })?;
        let current_queued_forms = api.get_sentinel_id_pk_forms().await?;
        let maybe_our_form = current_queued_forms
            .iter()
            .find(|f| f.sentinel_id == sentinel_id);

        Ok(maybe_our_form
            .map(|form| form.form.not_valid_after() >= now)
            .unwrap_or(false))
    }

    async fn create_and_upload_form(
        api: &ApiClient,
        candidate: &Self::Unregistered,
        latest_key_pair: &Self,
        now: DateTime<Utc>,
    ) -> Result<()> {
        let form_for_identity_api =
            RotateSentinelIdPublicKeyForm::new(candidate.public_key(), latest_key_pair, now)?;
        let form_for_api =
            RotateSentinelIdPublicKeyFormForm::new(form_for_identity_api, latest_key_pair, now)?;
        api.post_rotate_sentinel_id_pk_form(form_for_api).await?;
        Ok(())
    }

    fn key_type_name() -> &'static str {
        "sentinel identity"
    }

    fn rotate_after() -> chrono::Duration {
        SENTINEL_ID_KEY_ROTATE_AFTER
    }

    fn poll_iterations() -> u64 {
        SENTINEL_ID_KEY_POLL_ITERATIONS
    }

    fn poll_sleep_duration() -> Duration {
        SENTINEL_ID_KEY_POLL_SLEEP_DURATION
    }

    async fn should_attempt_rotation(vault: &JournalistVault) -> Result<bool> {
        if vault.sentinel_id().await?.is_none() {
            tracing::warn!(
                "No valid sentinel identity found in vault, cannot rotate sentinel ID keys"
            );
            return Ok(false);
        }
        Ok(true)
    }
}
