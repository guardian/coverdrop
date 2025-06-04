use std::path::PathBuf;

use chrono::{DateTime, Utc};
use common::{
    api::{api_client::ApiClient, models::journalist_id::JournalistIdentity},
    protocol::keys::{load_anchor_org_pks, load_journalist_provisioning_key_pairs, LatestKey},
};
use reqwest::Url;

#[allow(clippy::too_many_arguments)]
pub async fn update_journalist(
    api_url: Url,
    journalist_id: JournalistIdentity,
    display_name: Option<String>,
    sort_name: Option<String>,
    is_desk: Option<bool>,
    description: Option<String>,
    keys_path: PathBuf,
    now: DateTime<Utc>,
) -> anyhow::Result<()> {
    let api_client = ApiClient::new(api_url);
    let org_pks = load_anchor_org_pks(&keys_path, now)?;

    let journalist_provisioning_key_pairs =
        load_journalist_provisioning_key_pairs(&keys_path, &org_pks, now)?;

    let latest_journalist_provisioning_key_pair =
        journalist_provisioning_key_pairs.latest_key_required()?;

    api_client
        .patch_journalist(
            journalist_id,
            display_name,
            sort_name,
            is_desk,
            description,
            latest_journalist_provisioning_key_pair,
            now,
        )
        .await?;

    Ok(())
}
