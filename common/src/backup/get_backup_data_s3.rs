use crate::{
    api::models::journalist_id::JournalistIdentity,
    aws::s3::client::S3Client,
    backup::constants::{S3_META_BACKUP_DATA_SIGNATURE, S3_META_SIGNED_WITH},
    clap::Stage,
    crypto::{keys::untrusted::signing::UntrustedSignedPublicSigningKey, Signature},
    protocol::{
        backup::get_backup_bucket_name,
        backup_data::{BackupDataBytes, BackupDataWithSignature},
        roles::JournalistId,
    },
};
use anyhow::{anyhow, Context};
use itertools::Itertools;

/// This function gets the latest journalist backup (by insert order) from s3
/// for the supplied journalist identity.
pub async fn get_latest_journalist_backup_from_s3(
    s3_client: &S3Client,
    stage: &Stage,
    journalist_id: &JournalistIdentity,
) -> anyhow::Result<BackupDataWithSignature> {
    let bucket_name = get_backup_bucket_name(stage);
    let journalist_backups = s3_client
        .list_objects(&bucket_name, &journalist_id.to_string())
        .await?;

    if journalist_backups.is_empty() {
        anyhow::bail!("No backups found for journalist id: {}", journalist_id);
    }

    // Find the latest backup by last modified date
    let file_name = journalist_backups
        .iter()
        .filter_map(|it| match (it.last_modified(), it.key()) {
            (Some(modified), Some(key)) => Some((modified, key)),
            _ => None,
        })
        .sorted_by(|a, b| a.0.cmp(b.0))
        .map(|(_, key)| key)
        .next_back();

    if let Some(file_name) = file_name {
        let backup_file_output = s3_client.get_object(&bucket_name, file_name).await?;

        // Extract metadata from the S3 object to reconstruct BackupDataWithSignature.
        // The body contains the raw backup_data_bytes (CBOR); the signature and signing key
        // are stored as S3 object metadata headers.
        let metadata = backup_file_output
            .metadata()
            .context("No metadata on backup S3 object")?;

        let signature_json = metadata
            .get(S3_META_BACKUP_DATA_SIGNATURE)
            .context("Missing backup-data-signature metadata")?;
        let signed_with_json = metadata
            .get(S3_META_SIGNED_WITH)
            .context("Missing signed-with metadata")?;

        let backup_data_signature: Signature<BackupDataBytes> =
            serde_json::from_str(signature_json)
                .context("deserialize backup-data-signature metadata")?;

        let signed_with: UntrustedSignedPublicSigningKey<JournalistId> =
            serde_json::from_str(signed_with_json).context("deserialize signed-with metadata")?;

        let backup_bytes = backup_file_output.body.collect().await?;
        let backup_data_bytes = BackupDataBytes(backup_bytes.into_bytes().to_vec());

        let retrieved_signed_backup_data =
            BackupDataWithSignature::new(backup_data_bytes, backup_data_signature, signed_with)?;

        Ok(retrieved_signed_backup_data)
    } else {
        Err(anyhow!("Failed to get filename from s3"))
    }
}
