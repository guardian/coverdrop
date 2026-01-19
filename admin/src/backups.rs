use anyhow::Context;
use chrono::{DateTime, Utc};
use common::api::api_client::ApiClient;
use common::api::models::journalist_id::JournalistIdentity;
use common::api::models::untrusted_keys_and_journalist_profiles::UntrustedKeysAndJournalistProfiles;
use common::aws::s3::client::S3Client;
use common::backup::get_backup_data_s3::get_latest_journalist_backup_from_s3;
use common::clap::Stage;
use common::protocol::backup::{
    coverup_finish_restore_step, coverup_initiate_restore_step, BackupRestorationInProgress,
    WrappedSecretShare,
};
use common::protocol::backup_data::BackupDataWithSignature;
use common::protocol::keys::{
    load_anchor_org_pks, load_backup_id_key_pairs, load_backup_msg_key_pairs,
};
use common::time;
use common::time::now;
use log::{debug, info};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use time::format_timestamp_for_filename;
use tokio::fs;

/// Bundle structure for the response step of backup restoration.
/// See `backup_initiate_restore_submit` function.
#[derive(Serialize, Deserialize)]
pub struct BackupInitiateRestoreResponseBundle {
    pub journalist_id: JournalistIdentity,
    pub signed_backup_data: BackupDataWithSignature,
    pub hierarchy: UntrustedKeysAndJournalistProfiles,
}

/// Step 1: Retrieves backup data from S3 and key hierarchy from the API (online).
/// This step should be run on an online machine, and the response bundle
/// should be transferred to the air-gapped machine.
pub async fn backup_initiate_restore(
    api_url: Url,
    s3_client: &S3Client,
    stage: &Stage,
    output_dir: &Path,
    journalist_id: &JournalistIdentity,
) -> anyhow::Result<PathBuf> {
    let api_client = ApiClient::new(api_url);
    debug!(
        "Initiating backup restore for journalist '{}'...",
        journalist_id
    );

    let signed_backup_data = get_latest_journalist_backup_from_s3(s3_client, stage, journalist_id)
        .await
        .with_context(|| {
            format!(
                "Failed to download backup data from S3 (stage={:?}) for journalist '{}'",
                stage, journalist_id
            )
        })?;

    // Retrieve the key hierarchy from the API
    let hierarchy = api_client
        .get_public_keys()
        .await
        .with_context(|| "Failed to fetch public keys and journalist profiles")?;

    // Create the response bundle
    let response_bundle = BackupInitiateRestoreResponseBundle {
        journalist_id: journalist_id.clone(),
        signed_backup_data,
        hierarchy,
    };

    // Save the response bundle to disk
    let output_file = output_dir.join(format!(
        "restore-{}-{}.response-bundle",
        journalist_id,
        format_timestamp_for_filename(now())
    ));
    fs::write(&output_file, serde_json::to_string(&response_bundle)?).await?;
    info!("Wrote response bundle to disk: {:?}", output_file);

    Ok(output_file)
}

/// Step 2: Finalizes the restore process (offline, air-gapped).
/// Verifies the backup data, decrypts it, and creates encrypted secret shares.
/// Returns the path to the in-progress bundle file and a vector of encrypted secret
/// shares that are to be shared with the recovery contacts.
pub async fn backup_initiate_restore_finalize(
    bundle_response_path: &Path,
    keys_dir: PathBuf,
    output_dir: &Path,
    now: DateTime<Utc>,
) -> anyhow::Result<(PathBuf, Vec<PathBuf>)> {
    // Load the response bundle
    let response_bundle =
        load_bundle::<BackupInitiateRestoreResponseBundle>(bundle_response_path).await?;

    debug!(
        "Restoring backup bundle {} for journalist '{}'...",
        bundle_response_path.display(),
        response_bundle.journalist_id
    );

    // Load keys and verify the backup data
    let org_pks = load_anchor_org_pks(&keys_dir, now)?;
    if org_pks.is_empty() {
        anyhow::bail!(
            "No organization public keys found in '{}'. Cannot verify backup.",
            keys_dir.display()
        );
    }
    debug!("Found {} organization public key(s)", org_pks.len());

    let backup_id_key_pairs = load_backup_id_key_pairs(&keys_dir, &org_pks, now)?;
    if backup_id_key_pairs.is_empty() {
        anyhow::bail!(
            "No backup identity key pairs found in '{}'. Cannot decrypt backup.",
            keys_dir.display()
        );
    }
    debug!(
        "Found {} backup identity key pair(s)",
        backup_id_key_pairs.len()
    );

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
    let backup_msg_key_pairs = load_backup_msg_key_pairs(&keys_dir, &backup_id_key_pairs, now)?;
    if backup_msg_key_pairs.is_empty() {
        anyhow::bail!(
            "No backup messaging key pairs found in '{}'. Cannot decrypt backup shares.",
            keys_dir.display()
        );
    }
    debug!(
        "Found {} backup messaging key pair(s)",
        backup_msg_key_pairs.len()
    );

    let restoration_in_progress = coverup_initiate_restore_step(
        expected_journalist_identity,
        response_bundle.signed_backup_data.clone(),
        &journalist_id_key,
        &backup_msg_key_pairs,
        now,
    )
    .with_context(|| "Failed to complete initiate restore step")?;

    // Save the in-progress bundle to disk
    let output_file = output_dir.join(format!(
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
        let share_file = output_dir.join(format!(
            "restore-{}-{}-share-{}-{}.recovery-share.txt",
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
    keys_dir: &Path,
    shares: Vec<WrappedSecretShare>,
    now: DateTime<Utc>,
) -> anyhow::Result<PathBuf> {
    // Load the in-progress bundle from disk
    let in_progress_bundle =
        load_bundle::<BackupRestorationInProgress>(in_progress_bundle_path).await?;

    debug!(
        "Completing restore with bundle {} for journalist '{}'...",
        in_progress_bundle_path.display(),
        in_progress_bundle.journalist_identity
    );

    // Load the journalist's backup message key pairs to decrypt the shares
    let org_pks = load_anchor_org_pks(keys_dir, now)?;
    if org_pks.is_empty() {
        anyhow::bail!(
            "No organization public keys found in '{}'. Cannot verify backup.",
            keys_dir.display()
        );
    }
    debug!("Found {} organization public key(s)", org_pks.len());

    let backup_id_key_pairs = load_backup_id_key_pairs(keys_dir, &org_pks, now)?;
    if backup_id_key_pairs.is_empty() {
        anyhow::bail!(
            "No backup identity key pairs found in '{}'. Cannot decrypt backup.",
            keys_dir.display()
        );
    }
    debug!(
        "Found {} backup identity key pair(s)",
        backup_id_key_pairs.len()
    );

    let backup_msg_key_pairs = load_backup_msg_key_pairs(keys_dir, &backup_id_key_pairs, now)?;
    if backup_msg_key_pairs.is_empty() {
        anyhow::bail!(
            "No backup messaging key pairs found in '{}'. Cannot decrypt backup shares.",
            keys_dir.display()
        );
    }
    debug!(
        "Found {} backup messaging key pair(s)",
        backup_msg_key_pairs.len()
    );

    debug!("Found {} recovery share(s)", shares.len());

    let k = 1;
    let restored_encrypted_vault =
        coverup_finish_restore_step(in_progress_bundle.clone(), shares, &backup_msg_key_pairs, k)
            .with_context(|| "Failed to complete restore step")?;

    // Save the restored encrypted vault to disk
    let output_file = restore_vault_in_dir.join(format!(
        "{}-restored-{}.vault",
        in_progress_bundle.journalist_identity,
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
