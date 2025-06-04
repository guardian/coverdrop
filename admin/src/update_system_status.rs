use std::path::Path;

use chrono::{DateTime, Utc};
use common::{
    api::{
        api_client::ApiClient,
        models::general::{StatusEvent, SystemStatus},
    },
    protocol::keys::{load_anchor_org_pks, LatestKey},
    system::keys::load_admin_key_pair,
};

pub async fn update_system_status(
    keys_path: impl AsRef<Path>,
    api_client: &ApiClient,
    status: SystemStatus,
    description: String,
    now: DateTime<Utc>,
) -> anyhow::Result<()> {
    let org_pk = load_anchor_org_pks(&keys_path, now)?;
    let admin_key_pair =
        load_admin_key_pair(&keys_path, &org_pk, now)?.into_latest_key_required()?;

    let status = StatusEvent::new(status, description, now);
    api_client
        .post_status_event(status, &admin_key_pair, now)
        .await?;
    Ok(())
}
