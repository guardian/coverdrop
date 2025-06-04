use std::path::Path;

use common::crypto::keys::serde::StorableKeyMaterial;

use super::{
    provisioning_key_pairs_bundle::{
        CoverNodeProvisioningKeyPairBundle, JournalistProvisioningKeyPairBundle,
    },
    read_bundle_from_disk, COVERNODE_PROVISIONING_KEY_PAIR_BUNDLE_FILENAME,
    JOURNALIST_PROVISIONING_KEY_PAIR_BUNDLE_FILENAME,
};

pub async fn copy_identity_api_key_pairs(
    bundle_directory_path: impl AsRef<Path>,
    target_directory: impl AsRef<Path>,
) -> anyhow::Result<()> {
    if !target_directory.as_ref().is_dir() {
        anyhow::bail!("Target path is not a directory");
    }

    let journalist_provisioning_key_pair_bundle_path = bundle_directory_path
        .as_ref()
        .join(JOURNALIST_PROVISIONING_KEY_PAIR_BUNDLE_FILENAME);

    let journalist_provisioning_key_pair_bundle =
        read_bundle_from_disk::<JournalistProvisioningKeyPairBundle>(
            journalist_provisioning_key_pair_bundle_path,
        )?;

    let covernode_provisioning_key_pair_bundle_path = bundle_directory_path
        .as_ref()
        .join(COVERNODE_PROVISIONING_KEY_PAIR_BUNDLE_FILENAME);

    let covernode_provisioning_key_pair_bundle = read_bundle_from_disk::<
        CoverNodeProvisioningKeyPairBundle,
    >(covernode_provisioning_key_pair_bundle_path)?;

    // It would be nice to actually verify these are valid rather than blindly copying them
    journalist_provisioning_key_pair_bundle
        .journalist_provisioning_key_pair
        .save_to_disk(&target_directory)?;

    covernode_provisioning_key_pair_bundle
        .covernode_provisioning_key_pair
        .save_to_disk(&target_directory)?;

    Ok(())
}
