use std::{
    fs,
    path::{Path, PathBuf},
};

use common::system::keys::{AdminKeyPair, UntrustedAdminKeyPair};
use serde::{Deserialize, Serialize};

use crate::ceremony::ADMIN_KEY_PAIR_BUNDLE_FILENAME;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminKeyPairBundle {
    pub admin_key_pair: UntrustedAdminKeyPair,
}

/// Saves the admin key pair which should be securely shared with developers
pub fn save_admin_key_pair_bundle(
    output_directory: impl AsRef<Path>,
    admin_key_pair: &AdminKeyPair,
) -> anyhow::Result<PathBuf> {
    assert!(output_directory.as_ref().is_dir());

    let bundle = AdminKeyPairBundle {
        admin_key_pair: admin_key_pair.to_untrusted(),
    };

    let path = output_directory
        .as_ref()
        .join(ADMIN_KEY_PAIR_BUNDLE_FILENAME);

    fs::write(&path, serde_json::to_string_pretty(&bundle)?)?;

    Ok(path)
}
