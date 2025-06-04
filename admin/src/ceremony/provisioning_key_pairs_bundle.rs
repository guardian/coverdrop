use std::{
    fs,
    path::{Path, PathBuf},
};

use common::protocol::keys::{
    CoverNodeProvisioningKeyPair, JournalistProvisioningKeyPair,
    UntrustedCoverNodeProvisioningKeyPair, UntrustedJournalistProvisioningKeyPair,
};
use serde::{Deserialize, Serialize};

use crate::ceremony::{
    COVERNODE_PROVISIONING_KEY_PAIR_BUNDLE_FILENAME,
    JOURNALIST_PROVISIONING_KEY_PAIR_BUNDLE_FILENAME,
};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JournalistProvisioningKeyPairBundle {
    pub journalist_provisioning_key_pair: UntrustedJournalistProvisioningKeyPair,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoverNodeProvisioningKeyPairBundle {
    pub covernode_provisioning_key_pair: UntrustedCoverNodeProvisioningKeyPair,
}

/// Saves journalist provisioning key pair, which is
/// used by the identity service to rotate keys, and by editorial administrators
/// to create new journalists
pub fn save_journalist_provisioning_bundle(
    output_directory: impl AsRef<Path>,
    journalist_provisioning_key_pair: &JournalistProvisioningKeyPair,
) -> anyhow::Result<PathBuf> {
    assert!(output_directory.as_ref().is_dir());

    let bundle = JournalistProvisioningKeyPairBundle {
        journalist_provisioning_key_pair: journalist_provisioning_key_pair.to_untrusted(),
    };

    let path = output_directory
        .as_ref()
        .join(JOURNALIST_PROVISIONING_KEY_PAIR_BUNDLE_FILENAME);

    fs::write(&path, serde_json::to_string_pretty(&bundle)?)?;

    Ok(path)
}

/// Saves CoverNode provisioning key pair, which is
/// used by the identity service to rotate keys
pub fn save_covernode_provisioning_bundle(
    output_directory: impl AsRef<Path>,
    covernode_provisioning_key_pair: &CoverNodeProvisioningKeyPair,
) -> anyhow::Result<PathBuf> {
    assert!(output_directory.as_ref().is_dir());

    let bundle = CoverNodeProvisioningKeyPairBundle {
        covernode_provisioning_key_pair: covernode_provisioning_key_pair.to_untrusted(),
    };

    let path = output_directory
        .as_ref()
        .join(COVERNODE_PROVISIONING_KEY_PAIR_BUNDLE_FILENAME);

    fs::write(&path, serde_json::to_string_pretty(&bundle)?)?;

    Ok(path)
}
