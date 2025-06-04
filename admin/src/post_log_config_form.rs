use std::path::PathBuf;

use common::{
    clients::handle_response,
    protocol::keys::{load_anchor_org_pks, LatestKey},
    system::{forms::PostLogConfigForm, keys::load_admin_key_pair},
    time,
};
use reqwest::Url;

pub async fn post_log_config_form(
    service_url: Url,
    keys_path: PathBuf,
    rust_log_directive: String,
) -> anyhow::Result<()> {
    let now = time::now();

    let anchor_org_pks = load_anchor_org_pks(&keys_path, now)?;
    let admin_key_pairs = load_admin_key_pair(&keys_path, &anchor_org_pks, now)?;

    let latest_admin_key_pair = admin_key_pairs.latest_key_required()?;

    let form = PostLogConfigForm::new(rust_log_directive, latest_admin_key_pair, now)?;

    let http_client = reqwest::Client::new();

    let resp = http_client.post(service_url).json(&form).send().await?;

    handle_response(resp).await
}
