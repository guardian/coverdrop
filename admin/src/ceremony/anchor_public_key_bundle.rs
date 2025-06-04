use std::{
    fs,
    path::{Path, PathBuf},
};

use common::protocol::keys::{AnchorOrganizationPublicKey, UntrustedAnchorOrganizationPublicKey};
use serde::{Deserialize, Serialize};

use super::ANCHOR_ORGANIZATION_PUBLIC_KEY_BUNDLE_FILENAME;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnchorOrganizationPublicKeyBundle {
    pub anchor_org_pk: UntrustedAnchorOrganizationPublicKey,
}

/// Save the trusted organization public key which is then
/// used in multiple places to verify the rest of the key hierarchy
pub fn save_anchor_public_key_bundle(
    output_directory: impl AsRef<Path>,
    anchor_org_pk: &AnchorOrganizationPublicKey,
) -> anyhow::Result<PathBuf> {
    let bundle = AnchorOrganizationPublicKeyBundle {
        anchor_org_pk: anchor_org_pk.to_untrusted(),
    };

    let path = output_directory
        .as_ref()
        .join(ANCHOR_ORGANIZATION_PUBLIC_KEY_BUNDLE_FILENAME);

    fs::write(&path, serde_json::to_string_pretty(&bundle)?)?;

    Ok(path)
}
