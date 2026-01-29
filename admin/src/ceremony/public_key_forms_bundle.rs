use common::{
    api::forms::{
        PostAdminPublicKeyForm, PostBackupIdKeyForm, PostBackupMsgKeyForm,
        PostCoverNodeProvisioningPublicKeyForm, PostJournalistProvisioningPublicKeyForm,
    },
    backup::keys::BackupIdKeyPair,
    protocol::keys::{
        BackupMessagingKeyPair, CoverNodeProvisioningKeyPair, JournalistProvisioningKeyPair,
        OrganizationKeyPair,
    },
    system::keys::AdminKeyPair,
    time,
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::ceremony::PUBLIC_KEY_FORMS_BUNDLE_FILENAME;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicKeyFormsBundle {
    pub journalist_provisioning_pk_form: PostJournalistProvisioningPublicKeyForm,
    pub covernode_provisioning_pk_form: PostCoverNodeProvisioningPublicKeyForm,
    pub admin_pk_form: PostAdminPublicKeyForm,
    pub backup_id_pk_form: PostBackupIdKeyForm,
    pub backup_msg_pk_form: PostBackupMsgKeyForm,
}

/// Saves a collection of upload forms which can be used to upload the public keys
/// generated during the key ceremony to the API.
pub fn save_public_key_forms_bundle(
    output_directory: impl AsRef<Path>,
    org_key_pair: &OrganizationKeyPair,
    journalist_provisioning_key_pair: &JournalistProvisioningKeyPair,
    covernode_provisioning_key_pair: &CoverNodeProvisioningKeyPair,
    admin_key_pair: &AdminKeyPair,
    backup_id_key_pair: &BackupIdKeyPair,
    backup_msg_key_pair: &BackupMessagingKeyPair,
) -> anyhow::Result<PathBuf> {
    assert!(output_directory.as_ref().is_dir());

    let now = time::now();

    let journalist_provisioning_pk_form = PostJournalistProvisioningPublicKeyForm::new_for_bundle(
        journalist_provisioning_key_pair.public_key().to_untrusted(),
        org_key_pair,
        now,
    )?;

    let covernode_provisioning_pk_form = PostCoverNodeProvisioningPublicKeyForm::new_for_bundle(
        covernode_provisioning_key_pair.public_key().to_untrusted(),
        org_key_pair,
        now,
    )?;

    let admin_pk_form = PostAdminPublicKeyForm::new_for_bundle(
        admin_key_pair.public_key().to_untrusted(),
        org_key_pair,
        now,
    )?;

    let backup_id_pk_form = PostBackupIdKeyForm::new_for_bundle(
        backup_id_key_pair.public_key().to_untrusted(),
        org_key_pair,
        now,
    )?;

    let backup_msg_pk_form = PostBackupMsgKeyForm::new_for_bundle(
        backup_msg_key_pair.public_key().to_untrusted(),
        backup_id_key_pair,
        now,
    )?;

    let bundle = PublicKeyFormsBundle {
        journalist_provisioning_pk_form,
        covernode_provisioning_pk_form,
        admin_pk_form,
        backup_id_pk_form,
        backup_msg_pk_form,
    };

    let path = output_directory
        .as_ref()
        .join(PUBLIC_KEY_FORMS_BUNDLE_FILENAME);

    fs::write(&path, serde_json::to_string_pretty(&bundle)?)?;

    Ok(path)
}
