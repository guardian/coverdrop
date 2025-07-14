use std::path::Path;

use common::{
    api::{forms::PostCoverNodeIdPublicKeyForm, models::covernode_id::CoverNodeIdentity},
    protocol::keys::{
        generate_covernode_id_key_pair, load_anchor_org_pks, load_covernode_provisioning_key_pairs,
        LatestKey,
    },
    time,
};
use covernode_database::Database;

pub async fn generate_covernode_database(
    keys_path: impl AsRef<Path>,
    covernode_identity: CoverNodeIdentity,
    covernode_db_password: &str,
    output_path: impl AsRef<Path>,
) -> anyhow::Result<()> {
    let now = time::now();

    if !output_path.as_ref().is_dir() {
        anyhow::bail!("Output path is not a directory");
    }

    let output_path = output_path
        .as_ref()
        .join(format!("{covernode_identity}.db"));

    let anchor_org_pks = load_anchor_org_pks(&keys_path, now)?;
    if anchor_org_pks.is_empty() {
        anyhow::bail!(
            "No trusted organization public keys found in {}",
            keys_path.as_ref().display()
        );
    };

    let covernode_provisioning_key_pair =
        load_covernode_provisioning_key_pairs(&keys_path, &anchor_org_pks, now)?;

    let Some(covernode_provisioning_key_pair) = covernode_provisioning_key_pair.latest_key() else {
        anyhow::bail!(
            "No CoverNode provisioning key pairs found in {}",
            keys_path.as_ref().display()
        );
    };

    let db = Database::open(&output_path, covernode_db_password).await?;

    let covernode_id_key_pair =
        generate_covernode_id_key_pair(covernode_provisioning_key_pair, now);

    let form = PostCoverNodeIdPublicKeyForm::new_for_bundle(
        covernode_identity.clone(),
        covernode_id_key_pair.public_key().to_untrusted(),
        covernode_provisioning_key_pair,
        now,
    )?;

    db.insert_setup_bundle(&form, &covernode_id_key_pair, now)
        .await?;

    Ok(())
}
