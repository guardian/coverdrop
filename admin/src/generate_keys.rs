use std::{fs, path::Path};

use chrono::{DateTime, Utc};
use common::{
    api::{
        api_client::ApiClient,
        forms::{
            PostAdminPublicKeyForm, PostCoverNodeProvisioningPublicKeyForm,
            PostJournalistProvisioningPublicKeyForm, COVERNODE_PROVISIONING_KEY_FORM_FILENAME,
            JOURNALIST_PROVISIONING_KEY_FORM_FILENAME,
        },
        models::covernode_id::CoverNodeIdentity,
    },
    crypto::keys::serde::StorableKeyMaterial,
    protocol::{
        self,
        keys::{
            self, generate_covernode_id_key_pair, load_anchor_org_pks, load_covernode_id_key_pairs,
            load_covernode_provisioning_key_pairs, load_latest_org_key_pair, LatestKey,
        },
    },
    system::{self},
    time::{self},
};

/// Generates and saves a key pair file to disk, usually for use as a organization trust anchor
/// Returns the path to the created file
pub fn generate_organization_key_pair(
    path: impl AsRef<Path>,
    quiet: bool,
    now: DateTime<Utc>,
) -> anyhow::Result<()> {
    let org_key_pair = keys::generate_organization_key_pair(now);

    org_key_pair.to_untrusted().save_to_disk(&path)?;

    // For the org key pair we need to store the public keys separately
    // to the secret keys since we must distribute them to clients
    // as trusted public keys.

    org_key_pair
        .public_key()
        .to_untrusted()
        .save_to_disk(&path)?;

    if !quiet {
        println!(
            "🔐 Organization keys generated in {:?}",
            fs::canonicalize(&path).unwrap()
        );
    }

    Ok(())
}
pub async fn generate_admin_key_pair(
    keys_path: impl AsRef<Path>,
    api_client: ApiClient,
    do_not_upload_to_api: bool,
    quiet: bool,
    now: DateTime<Utc>,
) -> anyhow::Result<()> {
    let org_key_pair = load_latest_org_key_pair(&keys_path, now)?;

    let admin_key_pair = system::keys::generate_admin_key_pair(&org_key_pair, now);

    if !do_not_upload_to_api {
        let form = PostAdminPublicKeyForm::new(
            admin_key_pair.public_key().to_untrusted(),
            &org_key_pair,
            now,
        )?;

        api_client.post_admin_pk(form).await?;
    }

    admin_key_pair.to_untrusted().save_to_disk(&keys_path)?;

    if !quiet {
        println!(
            "🔐 System status key pair generated in {:?}",
            fs::canonicalize(keys_path).unwrap()
        );
    }

    Ok(())
}

pub async fn generate_journalist_provisioning_key_pair(
    keys_path: impl AsRef<Path>,
    quiet: bool,
    now: DateTime<Utc>,
) -> anyhow::Result<()> {
    let org_key_pair = load_latest_org_key_pair(&keys_path, now)?;

    let journalist_provisioning_key_pair =
        keys::generate_journalist_provisioning_key_pair(&org_key_pair, now);

    journalist_provisioning_key_pair
        .to_untrusted()
        .save_to_disk(&keys_path)?;

    if !quiet {
        println!(
            "🔐 Journalist provisioning key pair generated in {:?}",
            fs::canonicalize(&keys_path).unwrap()
        );
    }

    // TODO the form type should be in the type system, then we won't need to pass the file name to save_to_disk
    let form_path = keys_path
        .as_ref()
        .join(JOURNALIST_PROVISIONING_KEY_FORM_FILENAME);
    PostJournalistProvisioningPublicKeyForm::new(
        journalist_provisioning_key_pair.public_key().to_untrusted(),
        &org_key_pair,
        now,
    )?
    .save_to_disk(&form_path)?;

    if !quiet {
        println!(
            "🔐 Journalist provisioning key form saved to {:?}.",
            fs::canonicalize(&form_path).unwrap()
        );
        println!("Move this to an online machine and post it to the API using the 'post-journalist-provisioning-key-form' command WITHIN ONE HOUR!.");
    }

    Ok(())
}

pub async fn generate_covernode_provisioning_key_pair(
    keys_path: impl AsRef<Path>,
    quiet: bool,
    now: DateTime<Utc>,
) -> anyhow::Result<()> {
    let org_key_pair = load_latest_org_key_pair(&keys_path, now)?;

    let covernode_provisioning_key_pair =
        keys::generate_covernode_provisioning_key_pair(&org_key_pair, now);

    covernode_provisioning_key_pair
        .to_untrusted()
        .save_to_disk(&keys_path)?;

    if !quiet {
        println!(
            "🔐 CoverNode provisioning key pair generated in {:?}",
            fs::canonicalize(&keys_path).unwrap()
        );
    }

    // TODO the form type should be in the type system, then we won't need to pass the file name to save_to_disk
    let form_path = keys_path
        .as_ref()
        .join(COVERNODE_PROVISIONING_KEY_FORM_FILENAME);
    PostCoverNodeProvisioningPublicKeyForm::new(
        covernode_provisioning_key_pair.public_key().to_untrusted(),
        &org_key_pair,
        now,
    )?
    .save_to_disk(&form_path)?;

    if !quiet {
        println!(
            "🔐 CoverNode provisioning key form saved to {:?}.",
            fs::canonicalize(&form_path).unwrap()
        );
        println!("Move this to an online machine and post it to the API using the 'post-covernode-provisioning-key-form' command WITHIN ONE HOUR!.");
    }

    Ok(())
}

pub async fn generate_covernode_identity_key_pair(
    covernode_id: CoverNodeIdentity,
    keys_path: impl AsRef<Path>,
    api_client: ApiClient,
    do_not_upload_to_api: bool,
    quiet: bool,
    now: DateTime<Utc>,
) -> anyhow::Result<()> {
    let org_pks = load_anchor_org_pks(&keys_path, now)?;

    let covernode_provisioning_key_pairs =
        load_covernode_provisioning_key_pairs(&keys_path, &org_pks, time::now())?;

    let latest_covernode_provisioning_key_pair =
        covernode_provisioning_key_pairs.into_latest_key_required()?;

    let covernode_id_key_pair =
        generate_covernode_id_key_pair(&latest_covernode_provisioning_key_pair, now);

    if !do_not_upload_to_api {
        api_client
            .post_covernode_id_pk(
                &covernode_id,
                covernode_id_key_pair.public_key(),
                &latest_covernode_provisioning_key_pair,
                now,
            )
            .await?;
    }

    covernode_id_key_pair
        .to_untrusted()
        .save_to_disk(&keys_path)?;

    if !quiet {
        println!(
            "🔐 CoverNode identity key pair generated in {:?}",
            fs::canonicalize(keys_path).unwrap()
        );
    }

    Ok(())
}

pub async fn generate_covernode_messaging_key_pair(
    keys_path: impl AsRef<Path>,
    api_client: ApiClient,
    do_not_upload_to_api: bool,
    quiet: bool,
    now: DateTime<Utc>,
) -> anyhow::Result<()> {
    let org_pks = load_anchor_org_pks(&keys_path, now)?;

    let covernode_provisioning_key_pairs =
        load_covernode_provisioning_key_pairs(&keys_path, &org_pks, now)?;

    let covernode_id_key_pairs =
        load_covernode_id_key_pairs(&keys_path, &covernode_provisioning_key_pairs, now)?;

    let latest_covernode_id_key_pair = covernode_id_key_pairs.into_latest_key_required()?;

    let covernode_msg_key_pair =
        protocol::keys::generate_covernode_messaging_key_pair(&latest_covernode_id_key_pair, now);

    if !do_not_upload_to_api {
        api_client
            .post_covernode_msg_pk(
                covernode_msg_key_pair.public_key(),
                &latest_covernode_id_key_pair,
                now,
            )
            .await?;
    }

    covernode_msg_key_pair
        .to_untrusted()
        .save_to_disk(&keys_path)?;

    if !quiet {
        println!(
            "🔐 CoverNode messaging key pair generated in {:?}",
            fs::canonicalize(keys_path).unwrap()
        );
    }

    Ok(())
}
