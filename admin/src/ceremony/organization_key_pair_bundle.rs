use std::{
    fs,
    path::{Path, PathBuf},
};

use common::protocol::keys::{OrganizationKeyPair, UntrustedOrganizationKeyPair};
use serde::{Deserialize, Serialize};

use crate::ceremony::ORGANIZATION_KEY_PAIR_BUNDLE_FILENAME;

#[derive(Serialize, Deserialize)]
pub struct OrganizationKeyPairsBundle {
    pub org_key_pair: UntrustedOrganizationKeyPair,
}

/// The most sacred of all bundles.
///
/// Contains the organization key pair. Which can be
/// used to bootstrap an entire CoverDrop system.
pub fn save_organization_key_pair_bundle(
    output_directory: impl AsRef<Path>,
    org_key_pair: &OrganizationKeyPair,
) -> anyhow::Result<PathBuf> {
    assert!(output_directory.as_ref().is_dir());

    let bundle = OrganizationKeyPairsBundle {
        org_key_pair: org_key_pair.to_untrusted(),
    };

    let path = output_directory
        .as_ref()
        .join(ORGANIZATION_KEY_PAIR_BUNDLE_FILENAME);

    fs::write(&path, serde_json::to_string_pretty(&bundle)?)?;

    Ok(path)
}
