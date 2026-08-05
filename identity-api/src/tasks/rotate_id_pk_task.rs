use std::marker::PhantomData;

use async_trait::async_trait;
use chrono::Duration;
use common::{api::api_client::ApiClient, protocol::keys::LatestKey as _, task::Task, time};
use identity_api_database::Database;

use super::id_key_to_rotate::IdKeyToRotate;

pub struct RotateIdPublicKeysTask<K: IdKeyToRotate> {
    interval: Duration,
    api_client: ApiClient,
    database: Database,
    _marker: PhantomData<K>,
}

impl<K: IdKeyToRotate> RotateIdPublicKeysTask<K> {
    pub fn new(interval: Duration, api_client: ApiClient, database: Database) -> Self {
        Self {
            interval,
            api_client,
            database,
            _marker: PhantomData,
        }
    }
}

#[async_trait]
impl<K: IdKeyToRotate> Task for RotateIdPublicKeysTask<K> {
    fn name(&self) -> &'static str {
        K::task_name()
    }

    async fn run(&self) -> anyhow::Result<()> {
        let anchor_org_pks = self
            .database
            .select_anchor_organization_pks(time::now())
            .await?;

        let keys = self
            .api_client
            .get_public_keys()
            .await
            .map(|keys_and_profiles| {
                keys_and_profiles
                    .into_trusted(&anchor_org_pks, time::now())
                    .keys
            })?;

        let to_rotate = K::get_rotation_forms(&self.api_client).await?;

        for entry in &to_rotate {
            let Some((identity, new_pk_raw)) = K::verify_and_extract(entry, &keys, time::now())
            else {
                continue;
            };

            // Verify the identity from form verification matches the form metadata
            if &identity != K::form_identity(entry) {
                tracing::error!(
                    "{} ID for queued ID key rotation form does not match the ID of the owner of the form's signing key",
                    K::task_name()
                );
                continue;
            }

            // Has this key already been uploaded?
            // TODO should we skip this form if the key exists?
            K::check_for_existing_key(&keys, &identity, &new_pk_raw)?;

            // Everything ok with the form - sign the new key and post it to the API
            let journalist_provisioning_key_pair = self
                .database
                .select_journalist_provisioning_key_pairs(time::now())
                .await?
                .into_latest_key_required()?;

            K::sign_and_post(
                &self.api_client,
                identity,
                new_pk_raw,
                &journalist_provisioning_key_pair,
                time::now(),
            )
            .await?;
        }

        Ok(())
    }

    fn interval(&self) -> Duration {
        self.interval
    }
}
