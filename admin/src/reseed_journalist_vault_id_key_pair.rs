use std::path::Path;

use chrono::{DateTime, Utc};
use common::{
    api::{forms::PostJournalistIdPublicKeyForm, models::journalist_id::JournalistIdentity},
    protocol::{
        self,
        keys::{load_anchor_org_pks, load_journalist_provisioning_key_pairs, LatestKey},
    },
};
use journalist_vault::{JournalistVault, ReplacementStrategy};

pub async fn reseed_journalist_vault_id_key_pair(
    keys_path: impl AsRef<Path>,
    journalist_id: JournalistIdentity,
    vault: &JournalistVault,
    now: DateTime<Utc>,
) -> anyhow::Result<()> {
    let anchor_org_pks = load_anchor_org_pks(&keys_path, now)?;

    let journalist_provisioning_pks =
        load_journalist_provisioning_key_pairs(&keys_path, &anchor_org_pks, now)?;

    let latest_journalist_provisioning_key_pair =
        journalist_provisioning_pks.latest_key_required()?;

    let journalist_id_key_pair = protocol::keys::generate_journalist_id_key_pair(
        latest_journalist_provisioning_key_pair,
        now,
    );

    let pk_upload_form = PostJournalistIdPublicKeyForm::new(
        journalist_id,
        journalist_id_key_pair.public_key().to_untrusted(),
        false,
        latest_journalist_provisioning_key_pair,
        now,
    )?;

    vault
        .add_vault_setup_bundle(
            latest_journalist_provisioning_key_pair.public_key(),
            journalist_id_key_pair,
            pk_upload_form,
            None,
            ReplacementStrategy::Replace,
        )
        .await?;

    Ok(())
}
