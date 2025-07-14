use async_trait::async_trait;
use chrono::Duration;
use common::{
    api::{api_client::ApiClient, forms::PostJournalistIdPublicKeyForm},
    crypto::keys::public_key::PublicKey as _,
    protocol::keys::{sign_journalist_id_pk, LatestKey as _},
    task::Task,
    time,
};
use identity_api_database::Database;

pub struct RotateJournalistIdPublicKeysTask {
    interval: Duration,
    api_client: ApiClient,
    database: Database,
}

impl RotateJournalistIdPublicKeysTask {
    pub fn new(interval: Duration, api_client: ApiClient, database: Database) -> Self {
        Self {
            interval,
            api_client,
            database,
        }
    }
}

#[async_trait]
impl Task for RotateJournalistIdPublicKeysTask {
    fn name(&self) -> &'static str {
        "rotate_journalist_id_public_keys"
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

        let to_rotate = self.api_client.get_journalist_id_pk_forms().await?;

        for journalist_id_and_form in to_rotate {
            let form = journalist_id_and_form.form;

            // Who is the journalist who is rotating their key?
            //
            // The path for this endpoint uses the "me" keyword, which is a special identifier
            // for the journalist sending the request.
            //
            // This allows us to avoid the added complexity of checking that the journalist
            // who submitted the form is the same as the journalist in the URL.
            //
            // This allows us to side-step a potential Insecure Direct Object Reference vulnerability:
            // https://en.wikipedia.org/wiki/Insecure_direct_object_reference
            //
            // If the journalist exists, but someone is maliciously trying to rotate their key
            // the form signature will not be valid, unless they also have that journalist's
            // secret key - at which point all bets are off.
            let Some((journalist_id, verifying_pk)) =
                keys.find_journalist_id_pk_from_raw_ed25519_pk(form.signing_pk())
            else {
                tracing::error!("No journalist ID and verifying pk found for form signing pk");
                continue;
            };

            // This is a very weird invariant to violate. It means the API has somehow misattributed who uploaded the
            // pk rotation form.
            if *journalist_id != journalist_id_and_form.journalist_id {
                tracing::error!("Journalist ID for queued ID key rotation form does not match the ID of the owner of the form's signing key");
                continue;
            }

            //
            // Verification of the form
            //

            // Is the form signature valid?
            let Ok(verified_form) = form.to_verified_form_data(verifying_pk, time::now()) else {
                tracing::error!(
                    "Could not verify form data for {}'s key rotation form",
                    journalist_id
                );
                continue;
            };

            let new_pk = verified_form.new_pk.to_trusted();

            // Has this key already been uploaded?
            if let Some((existing_journalist_id, existing_journalist_id_pk)) =
                keys.find_journalist_id_pk_from_raw_ed25519_pk(&new_pk.key)
            {
                tracing::warn!(
                    "Journalist {} has attempted to upload a key that already exists: {}",
                    existing_journalist_id,
                    existing_journalist_id_pk.public_key_hex()
                );

                // This key already exists but is registered to a different journalist.
                // This should not happen.
                if existing_journalist_id != journalist_id {
                    anyhow::bail!(
                        "Key exists already but has been registered to a different journalist"
                    );
                }

                // The new key already exists in the API but the identity API does not know it's epoch
                // So we need to request the key and epoch from the API anyway
            }

            //
            // Everything is ok with our form! Let's rotate the key
            //

            let journalist_provisioning_key_pair = self
                .database
                .select_journalist_provisioning_key_pairs(time::now())
                .await?
                .into_latest_key_required()?;

            let signed_journalist_id_pk =
                sign_journalist_id_pk(new_pk, &journalist_provisioning_key_pair, time::now());

            tracing::debug!(
                "Signed new journalist id public key for {}: {}",
                journalist_id,
                &serde_json::to_string(&signed_journalist_id_pk.to_untrusted())
                    .unwrap_or_else(|e| format!("<failed to serialize: {e}>"))
            );

            let form = PostJournalistIdPublicKeyForm::new(
                journalist_id.clone(),
                signed_journalist_id_pk.to_untrusted(),
                true,
                &journalist_provisioning_key_pair,
                time::now(),
            )?;

            self.api_client.post_journalist_id_pk_form(form).await?;
        }

        Ok(())
    }

    fn interval(&self) -> Duration {
        self.interval
    }
}
