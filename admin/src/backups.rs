use anyhow::Context;
use chrono::{DateTime, Utc};
use common::api::api_client::ApiClient;
use common::api::forms::GetBackupDataForm;
use common::api::models::journalist_id::JournalistIdentity;
use common::api::models::untrusted_keys_and_journalist_profiles::UntrustedKeysAndJournalistProfiles;
use common::protocol::backup::{
    coverup_finish_restore_step, coverup_initiate_restore_step, BackupRestorationInProgress,
    WrappedSecretShare,
};
use common::protocol::backup_data::BackupDataWithSignature;
use common::protocol::keys::{
    load_anchor_org_pks, load_backup_id_key_pairs, load_backup_msg_key_pairs, LatestKey,
};
use common::time;
use common::time::now;
use log::info;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use time::format_timestamp_for_filename;
use tokio::fs;

/// Bundle structure for the prepare step of backup restoration.
/// See `backup_initiate_restore_prepare` function.
#[derive(Serialize, Deserialize)]
pub struct BackupInitiateRestorePrepareBundle {
    pub journalist_id: JournalistIdentity,
    pub form: GetBackupDataForm,
}

/// Bundle structure for the response step of backup restoration.
/// See `backup_initiate_restore_submit` function.
#[derive(Serialize, Deserialize)]
pub struct BackupInitiateRestoreResponseBundle {
    pub journalist_id: JournalistIdentity,
    pub signed_backup_data: BackupDataWithSignature,
    pub hierarchy: UntrustedKeysAndJournalistProfiles,
}

/// Step 1: Prepares a backup restore request bundle (offline, air-gapped).
/// Creates a bundle containing the journalist ID and backup data request form.
/// This bundle should be transferred to an online machine for submission.
pub async fn backup_initiate_restore_prepare(
    keys_path: PathBuf,
    journalist_id: JournalistIdentity,
    bundle_path: &Path,
    now: DateTime<Utc>,
) -> anyhow::Result<PathBuf> {
    // Load keys to create the backup data request form
    let org_pks = load_anchor_org_pks(&keys_path, now)?;
    let backup_id_key_pairs = load_backup_id_key_pairs(&keys_path, &org_pks, now)?;
    let form = GetBackupDataForm::new(
        journalist_id.clone(),
        &backup_id_key_pairs.clone().into_latest_key_required()?,
        now,
    )?;

    // Create the prepare bundle
    let prepare_bundle = BackupInitiateRestorePrepareBundle {
        journalist_id: journalist_id.clone(),
        form,
    };

    // Save the prepare bundle to disk
    let output_file = bundle_path.join(format!(
        "restore-{}-{}.prepare-bundle",
        journalist_id,
        format_timestamp_for_filename(now)
    ));
    fs::write(&output_file, serde_json::to_string(&prepare_bundle)?).await?;
    info!("Wrote prepare bundle to disk: {:?}", output_file);

    Ok(output_file)
}

/// Step 2: Submits the prepare bundle to the API (online).
/// Retrieves the backup data and key hierarchy from the API.
/// This step should be run on an online machine, and the response bundle
/// should be transferred back to the air-gapped machine.
pub async fn backup_initiate_restore_submit(
    bundle_path: &Path,
    api_url: Url,
    output_path: &Path,
) -> anyhow::Result<PathBuf> {
    // Load the prepare bundle
    let prepare_bundle = load_bundle::<BackupInitiateRestorePrepareBundle>(bundle_path).await?;

    let api_client = ApiClient::new(api_url);

    // Retrieve the backup data from the API
    let signed_backup_data = api_client
        .retrieve_backup(prepare_bundle.form)
        .await
        .with_context(|| {
            format!(
                "Failed to fetch backup data for journalist {:?}",
                prepare_bundle.journalist_id
            )
        })?;

    // Retrieve the key hierarchy from the API
    let hierarchy = api_client
        .get_public_keys()
        .await
        .with_context(|| "Failed to fetch public keys and journalist profiles")?;

    // Create the response bundle
    let response_bundle = BackupInitiateRestoreResponseBundle {
        journalist_id: prepare_bundle.journalist_id.clone(),
        signed_backup_data,
        hierarchy,
    };

    // Save the response bundle to disk
    let output_file = output_path.join(format!(
        "restore-{}-{}.response-bundle",
        prepare_bundle.journalist_id,
        format_timestamp_for_filename(now())
    ));
    fs::write(&output_file, serde_json::to_string(&response_bundle)?).await?;
    info!("Wrote response bundle to disk: {:?}", output_file);

    Ok(output_file)
}

/// Step 3: Finalizes the restore process (offline, air-gapped).
/// Verifies the backup data, decrypts it, and creates encrypted secret shares.
/// Returns the path to the in-progress bundle file and a vector of encrypted secret
/// shares that are to be shared with the recovery contacts.
pub async fn backup_initiate_restore_finalize(
    bundle_response_path: &Path,
    keys_path: PathBuf,
    output_path: &Path,
    now: DateTime<Utc>,
) -> anyhow::Result<(PathBuf, Vec<PathBuf>)> {
    // Load the response bundle
    let response_bundle =
        load_bundle::<BackupInitiateRestoreResponseBundle>(bundle_response_path).await?;

    // Load keys and verify the backup data
    let org_pks = load_anchor_org_pks(&keys_path, now)?;
    let backup_id_key_pairs = load_backup_id_key_pairs(&keys_path, &org_pks, now)?;

    let hierarchy = response_bundle.hierarchy.into_trusted(&org_pks, now);
    let journalist_id_key = hierarchy
        .keys
        .verify_journalist_id_key(
            response_bundle.signed_backup_data.signed_with().clone(),
            now,
        )
        .context("Failed to verify journalist identity key from backup data")?;

    // Check that the signing key matches the journalist identity associated with this key in the
    // public key hierarchy.
    let expected_journalist_identity = response_bundle.journalist_id.clone();
    let (journalist_identity_from_key, _) = hierarchy
        .keys
        .find_journalist_id_pk_from_raw_ed25519_pk(&journalist_id_key.clone().key)
        .context("Failed to find signing key in hierarchy")?;
    if *journalist_identity_from_key != expected_journalist_identity {
        return Err(anyhow::anyhow!(
            "Journalist identity from signing key does not match expected identity"
        ));
    }

    // Load the journalist's backup message key pairs and decrypt the backup data into the
    // in-progress bundle format
    let backup_msg_key_pairs = load_backup_msg_key_pairs(&keys_path, &backup_id_key_pairs, now)?;
    let restoration_in_progress = coverup_initiate_restore_step(
        expected_journalist_identity,
        response_bundle.signed_backup_data.clone(),
        &journalist_id_key,
        &backup_msg_key_pairs,
        now,
    )
    .with_context(|| "Failed to complete initiate restore step")?;

    // Save the in-progress bundle to disk
    let output_file = output_path.join(format!(
        "restore-{}-{}.recovery-in-progress",
        response_bundle.journalist_id,
        format_timestamp_for_filename(now)
    ));
    fs::write(
        &output_file,
        serde_json::to_string(&restoration_in_progress)?,
    )
    .await?;
    info!(
        "Wrote restoration in-progress bundle to disk: {:?}",
        output_file
    );

    // Save the encrypted secret shares to disk
    let mut encrypted_shares_files = Vec::new();
    for (i, (recipient_id, encrypted_share)) in restoration_in_progress
        .encrypted_shares
        .into_iter()
        .enumerate()
    {
        let share_file = output_path.join(format!(
            "restore-{}-{}-share-{}-{}.recovery-share",
            response_bundle.journalist_id,
            format_timestamp_for_filename(now),
            i + 1,
            recipient_id
        ));
        fs::write(&share_file, encrypted_share.to_base64_string())
            .await
            .with_context(|| "Failed to write wrapped secret share")?;
        info!("Wrote wrapped secret share to disk: {:?}", share_file);
        encrypted_shares_files.push(share_file);
    }

    Ok((output_file, encrypted_shares_files))
}

pub async fn backup_complete_restore(
    in_progress_bundle_path: &Path,
    restore_vault_in_dir: &Path,
    keys_path: &Path,
    shares: Vec<WrappedSecretShare>,
    now: DateTime<Utc>,
) -> anyhow::Result<PathBuf> {
    // Load the in-progress bundle from disk
    let in_progress_bundle =
        load_bundle::<BackupRestorationInProgress>(in_progress_bundle_path).await?;

    // Load the journalist's backup message key pairs to decrypt the shares
    let org_pks = load_anchor_org_pks(keys_path, now)?;
    let backup_id_key_pairs = load_backup_id_key_pairs(keys_path, &org_pks, now)?;
    let backup_msg_key_pairs = load_backup_msg_key_pairs(keys_path, &backup_id_key_pairs, now)?;

    let k = 1;
    let restored_encrypted_vault =
        coverup_finish_restore_step(in_progress_bundle, shares, &backup_msg_key_pairs, k)
            .with_context(|| "Failed to complete restore step")?;

    // Save the restored encrypted vault to disk
    let output_file = restore_vault_in_dir.join(format!(
        "restored-{}.vault",
        format_timestamp_for_filename(now)
    ));
    fs::write(&output_file, &restored_encrypted_vault).await?;
    info!("Wrote restored vault to disk: {:?}", output_file);

    Ok(output_file)
}

async fn load_bundle<T>(bundle_path: &Path) -> anyhow::Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let bundle: T = serde_json::from_slice(&fs::read(bundle_path).await?)
        .with_context(|| "Failed to read bundle from disk")?;
    Ok(bundle)
}
