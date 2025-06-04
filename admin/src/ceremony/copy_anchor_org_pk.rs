use std::path::Path;

use chrono::{DateTime, Utc};
use common::{crypto::keys::serde::StorableKeyMaterial, protocol::keys::anchor_org_pk};

use super::{
    anchor_public_key_bundle::AnchorOrganizationPublicKeyBundle, read_bundle_from_disk,
    ANCHOR_ORGANIZATION_PUBLIC_KEY_BUNDLE_FILENAME,
};

pub async fn copy_anchor_org_pk(
    bundle_directory_path: impl AsRef<Path>,
    target_directory: impl AsRef<Path>,
    now: DateTime<Utc>,
) -> anyhow::Result<()> {
    if !target_directory.as_ref().is_dir() {
        anyhow::bail!("Target path is not a directory");
    }

    let bundle = bundle_directory_path
        .as_ref()
        .join(ANCHOR_ORGANIZATION_PUBLIC_KEY_BUNDLE_FILENAME);

    let anchor_org_pk_bundle = read_bundle_from_disk::<AnchorOrganizationPublicKeyBundle>(bundle)?;

    let anchor_org_pk = anchor_org_pk(&anchor_org_pk_bundle.anchor_org_pk, now)?;

    // The org pk is valid, now can save it to target directory

    anchor_org_pk
        .to_untrusted()
        .save_to_disk(target_directory)?;

    Ok(())
}
