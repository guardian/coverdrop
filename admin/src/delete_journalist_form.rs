use std::{
    fs::File,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Utc};
use common::{
    api::{
        api_client::ApiClient, forms::DeleteJournalistForm,
        models::journalist_id::JournalistIdentity,
    },
    protocol::keys::{load_anchor_org_pks, load_journalist_provisioning_key_pairs, LatestKey},
};

pub async fn submit_delete_journalist_form(
    api_client: &ApiClient,
    form_path: impl AsRef<Path>,
) -> anyhow::Result<()> {
    let mut file = File::open(form_path)?;

    let form = serde_json::from_reader(&mut file)?;

    api_client.delete_journalist(&form).await?;

    Ok(())
}

pub async fn delete_journalist_form(
    keys_path: impl AsRef<Path>,
    journalist_id: &JournalistIdentity,
    output_path: impl AsRef<Path>,
    now: DateTime<Utc>,
) -> anyhow::Result<PathBuf> {
    let output_path = output_path.as_ref();

    if !output_path.is_dir() {
        anyhow::bail!("Output path is not a directory");
    }

    let org_pks = load_anchor_org_pks(&keys_path, now)?;

    let journalist_provisioning_key_pairs =
        load_journalist_provisioning_key_pairs(&keys_path, &org_pks, now)?;

    let latest_journalist_provisioning_key_pair =
        journalist_provisioning_key_pairs.into_latest_key_required()?;

    let form = DeleteJournalistForm::new(
        journalist_id.clone(),
        &latest_journalist_provisioning_key_pair,
        now,
    )?;

    let output_file_path = output_path.join(format!("delete_{}.form.json", &journalist_id));

    let mut file = File::create_new(&output_file_path)?;

    serde_json::to_writer(&mut file, &form)?;

    Ok(output_file_path)
}
