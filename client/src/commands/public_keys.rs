use chrono::{DateTime, Utc};
use common::{
    api::api_client::ApiClient, client::VerifiedKeysAndJournalistProfiles,
    protocol::keys::AnchorOrganizationPublicKey,
};

pub async fn get_public_keys(
    org_pks: &[AnchorOrganizationPublicKey],
    client: &ApiClient,
    now: DateTime<Utc>,
) -> anyhow::Result<VerifiedKeysAndJournalistProfiles> {
    Ok(client.get_public_keys().await?.into_trusted(org_pks, now))
}
