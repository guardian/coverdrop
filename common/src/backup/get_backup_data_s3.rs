use crate::{
    api::models::journalist_id::JournalistIdentity,
    aws::s3::client::S3Client,
    clap::Stage,
    protocol::{backup::get_backup_bucket_name, backup_data::BackupDataWithSignature},
};
use anyhow::anyhow;
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

        let backup_bytes = backup_file_output.body.collect().await?;

        let bytes = backup_bytes.into_bytes();
        let retrieved_signed_backup_data: BackupDataWithSignature = serde_json::from_slice(&bytes)?;

        Ok(retrieved_signed_backup_data)
    } else {
        Err(anyhow!("Failed to get filename from s3"))
    }
}
