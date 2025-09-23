use chrono::{DateTime, Utc};
use common::backup::keys::{
    verify_backup_id_pk, verify_backup_msg_pk, BackupIdPublicKey, BackupMsgPublicKey,
    UntrustedBackupIdPublicKey, UntrustedBackupMsgPublicKey,
};
use common::{
    crypto::keys::{signing::traits::PublicSigningKey, Ed25519PublicKey},
    protocol::keys::{anchor_org_pk, OrganizationPublicKey, UntrustedAnchorOrganizationPublicKey},
};
use serde_json::Value;
use sqlx::PgPool;

#[derive(Clone)]
pub struct BackupKeyQueries {
    pool: PgPool,
}

impl BackupKeyQueries {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert_backup_id_pk(
        &self,
        backup_pk: &BackupIdPublicKey,
        signing_pk: &OrganizationPublicKey,
    ) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;

        let org_pk_id = sqlx::query!(
            r#"
                SELECT id AS org_pk_id
                FROM organization_pks
                WHERE decode(pk_json #>>'{key}', 'hex') = $1
            "#,
            signing_pk.as_bytes()
        )
        .map(|row| row.org_pk_id)
        .fetch_one(&mut *tx)
        .await?;

        let backup_pk = backup_pk.to_untrusted();

        sqlx::query!(
            r#"
                INSERT INTO backup_id_pks (org_pk_id, not_valid_after, pk_json)
                    VALUES ($1, $2, $3)
            "#,
            org_pk_id,
            backup_pk.not_valid_after,
            serde_json::to_value(&backup_pk)?,
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn find_backup_signing_pk_from_ed25519_pk(
        &self,
        candidate_pk: &Ed25519PublicKey,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Option<BackupIdPublicKey>> {
        let mut connection = self.pool.acquire().await?;

        sqlx::query!(
            r#"
            SELECT
                backup_id_pks.pk_json AS "id_pk: Value",
                organization_pks.pk_json  AS "org_pk: Value"
            FROM backup_id_pks
            LEFT JOIN organization_pks
                ON backup_id_pks.org_pk_id = organization_pks.id
            WHERE decode(backup_id_pks.pk_json #>>'{key}', 'hex') = $1
            "#,
            candidate_pk.as_bytes()
        )
        .fetch_optional(&mut *connection)
        .await?
        .map(|row| {
            let org_pk =
                serde_json::from_value::<UntrustedAnchorOrganizationPublicKey>(row.org_pk)?;
            let org_pk = anchor_org_pk(&org_pk, now)?.to_non_anchor();

            let backup_signing_pk =
                serde_json::from_value::<UntrustedBackupIdPublicKey>(row.id_pk)?;
            let backup_signing_pk = verify_backup_id_pk(&backup_signing_pk, &org_pk, now)?;
            anyhow::Ok(backup_signing_pk)
        })
        .transpose()
    }

    pub async fn insert_backup_encryption_pk(
        &self,
        backup_msg_pk: &BackupMsgPublicKey,
        backup_id_pk: &BackupIdPublicKey,
    ) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;

        let backup_id_pk_id = sqlx::query!(
            r#"
                SELECT id AS backup_id_pk_id
                FROM backup_id_pks
                WHERE decode(pk_json #>>'{key}', 'hex') = $1
            "#,
            backup_id_pk.as_bytes()
        )
        .map(|row| row.backup_id_pk_id)
        .fetch_one(&mut *tx)
        .await?;

        let backup_pk = backup_msg_pk.to_untrusted();

        sqlx::query!(
            r#"
                INSERT INTO backup_msg_pks (backup_id_pk_id, not_valid_after, pk_json)
                    VALUES ($1, $2, $3)
            "#,
            backup_id_pk_id,
            backup_pk.not_valid_after,
            serde_json::to_value(&backup_pk)?,
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn find_backup_encryption_pk_from_x25519_pk(
        &self,
        candidate_pk: &Ed25519PublicKey,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Option<BackupMsgPublicKey>> {
        let mut connection = self.pool.acquire().await?;

        sqlx::query!(
            r#"
            SELECT
                backup_msg_pks.pk_json AS "backup_msg_pk: Value",
                backup_id_pks.pk_json AS "backup_id_pk: Value",
                organization_pks.pk_json  AS "org_pk: Value"
            FROM backup_msg_pks
            LEFT JOIN backup_id_pks
                ON backup_msg_pks.backup_id_pk_id = backup_id_pks.id
            LEFT JOIN organization_pks
                ON backup_id_pks.org_pk_id = organization_pks.id
            WHERE decode(backup_id_pks.pk_json #>>'{key}', 'hex') = $1
            "#,
            candidate_pk.as_bytes()
        )
        .fetch_optional(&mut *connection)
        .await?
        .map(|row| {
            let org_pk =
                serde_json::from_value::<UntrustedAnchorOrganizationPublicKey>(row.org_pk)?;
            let org_pk = anchor_org_pk(&org_pk, now)?.to_non_anchor();

            let backup_id_pk =
                serde_json::from_value::<UntrustedBackupIdPublicKey>(row.backup_id_pk)?;

            let signed_backup_id_pk = verify_backup_id_pk(&backup_id_pk, &org_pk, now)?;

            let backup_msg_pk =
                serde_json::from_value::<UntrustedBackupMsgPublicKey>(row.backup_msg_pk)?;

            verify_backup_msg_pk(&backup_msg_pk, &signed_backup_id_pk, now)
        })
        .transpose()
    }
}
