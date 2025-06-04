use std::path::Path;

use common::crypto::keys::serde::StorableKeyMaterial;

use super::{read_bundle_from_disk, AdminKeyPairBundle, ADMIN_KEY_PAIR_BUNDLE_FILENAME};

pub async fn copy_admin_key_pair(
    bundle_directory_path: impl AsRef<Path>,
    target_directory: impl AsRef<Path>,
) -> anyhow::Result<()> {
    if !target_directory.as_ref().is_dir() {
        anyhow::bail!("Target path is not a directory");
    }

    let bundle = bundle_directory_path
        .as_ref()
        .join(ADMIN_KEY_PAIR_BUNDLE_FILENAME);

    let admin_key_pair_bundle = read_bundle_from_disk::<AdminKeyPairBundle>(bundle)?;

    admin_key_pair_bundle
        .admin_key_pair
        .save_to_disk(target_directory)?;

    Ok(())
}
