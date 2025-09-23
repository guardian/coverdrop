pub(crate) use common::api::models::journalist_id::JournalistIdentity;
use common::crypto::Signature;
use common::protocol::backup_data::{BackupDataBytes, BackupDataWithSignature, VerifiedBackupData};
use common::protocol::keys::OrganizationPublicKeyFamilyList;
use common::time::now;
use serde_json::Value;
use sqlx::PgPool;

#[derive(Clone)]
pub struct BackupDataQueries {
    pool: PgPool,
}

impl BackupDataQueries {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn store_backup_data(
        &self,
        backup_data: &VerifiedBackupData,
    ) -> anyhow::Result<bool> {
        let mut tx = self.pool.begin().await?;

        let journalist_id_pk_id = sqlx::query!(
            r#"
                SELECT id AS journalist_id_pk
                FROM journalist_id_pks
                WHERE decode(pk_json #>>'{key}', 'hex') = $1
            "#,
            &backup_data.signed_with.key.to_bytes()
        )
        .map(|row| row.journalist_id_pk)
        .fetch_one(&mut *tx)
        .await?;

        let result = sqlx::query!(
            r#"
                INSERT INTO backups (created_at, data, signature, signing_key_json, journalist_id_pk_id)
                VALUES ($1, $2, $3, $4, $5)
            "#,
            &backup_data.backup_data()?.created_at,
            &backup_data.backup_data_bytes.as_bytes(),
            &backup_data.backup_data_signature.to_bytes(),
            serde_json::to_value(&backup_data.signed_with.to_untrusted())?,
            journalist_id_pk_id
        )
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        if result.rows_affected() != 1 {
            tracing::info!("Backup data already exists, not inserting duplicate");
            return Ok(false);
        }

        Ok(true)
    }

    pub async fn get_latest_backup_data(
        &self,
        keys: OrganizationPublicKeyFamilyList,
        journalist_id: &JournalistIdentity,
    ) -> anyhow::Result<VerifiedBackupData> {
        let mut tx = self.pool.begin().await?;

        // Find the backup data for the given journalist_id
        let backup_data = sqlx::query!(
            r#"
                SELECT
                    data AS "data:Vec<u8>",
                    signature AS "signature:Vec<u8>",
                    signing_key_json AS "signing_key_json:Value",
                    journalist_id_pk_id AS "journalist_id_pk_id: i32"
                FROM backups
                JOIN journalist_id_pks ON backups.journalist_id_pk_id = journalist_id_pks.id
                WHERE journalist_id_pks.journalist_profile_id = $1
                ORDER BY created_at DESC
                LIMIT 1
            "#,
            journalist_id
        )
        .fetch_one(&mut *tx)
        .await?;

        let signed_backup_data = BackupDataWithSignature::new(
            BackupDataBytes(backup_data.data),
            Signature::<BackupDataBytes>::from_vec_unchecked(backup_data.signature),
            serde_json::from_value(backup_data.signing_key_json)?,
        )?;

        // Retrieve the journalist_id_pk from the key hierarchy used to sign the backup data
        let (journalist_id, journalist_id_key) = keys
            .find_journalist_id_pk_from_raw_ed25519_pk(&signed_backup_data.signed_with().key)
            .ok_or_else(|| anyhow::anyhow!("Signing key not found"))?;

        let verified_backup_data = signed_backup_data.to_verified(journalist_id_key, now())?;

        // Verify that the journalist_id from the backup data matches the requested journalist_id
        if verified_backup_data.backup_data()?.journalist_identity != *journalist_id {
            return Err(anyhow::anyhow!("Journalist ID does not match signing key"));
        }

        Ok(verified_backup_data)
    }
}
