use std::path::Path;

use chrono::{DateTime, Utc};
use common::{
    argon2_sqlcipher::Argon2SqlCipher,
    crypto::keys::public_key::PublicKey,
    protocol::keys::{
        anchor_org_pk, AnchorOrganizationPublicKey, CoverNodeProvisioningKeyPair,
        JournalistProvisioningKeyPair, UntrustedAnchorOrganizationPublicKey,
        UntrustedCoverNodeProvisioningKeyPair, UntrustedJournalistProvisioningKeyPair,
    },
};
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn open(path: impl AsRef<Path>, password: &str) -> anyhow::Result<Database> {
        let path = path.as_ref();
        tracing::info!("Opening DB: {}", path.display());

        // Note that this *MUST* be a default SQLite journaling mode (not WAL) because we
        // currently deploy the CoverNode to a system running NFS, which doesn't work well with
        // SQLite in WAL mode.
        let pool = if path.exists() {
            let database =
                Argon2SqlCipher::open_and_maybe_migrate_from_legacy(path, password).await?;
            database.into_sqlite_pool()
        } else {
            let database = Argon2SqlCipher::new(path, password).await?;
            database.into_sqlite_pool()
        };

        tracing::info!("Migrating DB: {}", path.display());
        sqlx::migrate!().run(&pool).await?;

        Ok(Database { pool })
    }

    // Organization Public Keys

    pub async fn insert_anchor_organization_pk(
        &self,
        anchor_org_pk: &AnchorOrganizationPublicKey,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let mut conn = self.pool.acquire().await?;

        let anchor_org_pk = anchor_org_pk.to_untrusted();
        let anchor_org_pk = serde_json::to_string(&anchor_org_pk)?;

        sqlx::query!(
            r#"
                INSERT INTO organization_public_keys
                (pk_json, created_at)
                VALUES
                (?1, ?2)
                ON CONFLICT DO NOTHING
            "#,
            anchor_org_pk,
            now,
        )
        .execute(&mut *conn)
        .await?;

        Ok(())
    }

    pub async fn select_anchor_organization_pks(
        &self,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Vec<AnchorOrganizationPublicKey>> {
        let mut conn = self.pool.acquire().await?;

        let rows = sqlx::query!(
            r#"
                SELECT
                    pk_json AS "pk_json: String"
                FROM organization_public_keys
            "#,
        )
        .fetch_all(&mut *conn)
        .await?;

        let pks = rows
            .into_iter()
            .map(|row| {
                let pk =
                    serde_json::from_str::<UntrustedAnchorOrganizationPublicKey>(&row.pk_json)?;

                anchor_org_pk(&pk, now)
            })
            .collect::<anyhow::Result<_>>()?;

        Ok(pks)
    }

    // Journalist Provisioning Keys

    pub async fn insert_journalist_provisioning_key_pair(
        &self,
        org_pk: &AnchorOrganizationPublicKey,
        key_pair: &JournalistProvisioningKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;
        let org_pk_hex = org_pk.public_key_hex();

        let org_pk_id = sqlx::query_scalar!(
            r#"
                SELECT
                    id AS "id: i64"
                FROM organization_public_keys
                WHERE json_extract(pk_json, '$.key') == ?1
            "#,
            org_pk_hex
        )
        .fetch_one(&mut *tx)
        .await?;

        let key_pair = key_pair.to_untrusted();
        let key_pair = serde_json::to_string(&key_pair)?;

        sqlx::query!(
            r#"
                INSERT INTO journalist_provisioning_key_pairs
                (organization_pk_id, key_pair_json, added_at)
                VALUES
                (?1, ?2, ?3)
                ON CONFLICT DO NOTHING
            "#,
            org_pk_id,
            key_pair,
            now,
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn select_journalist_provisioning_key_pairs(
        &self,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Vec<JournalistProvisioningKeyPair>> {
        let mut conn = self.pool.acquire().await?;

        let rows = sqlx::query!(
            r#"
                SELECT
                    key_pair_json AS "key_pair_json: String",
                    pk_json       AS "org_pk_json: String"
                FROM journalist_provisioning_key_pairs jp
                INNER JOIN organization_public_keys o
                    ON o.id = jp.organization_pk_id
            "#,
        )
        .fetch_all(&mut *conn)
        .await?;

        let key_pairs = rows
            .into_iter()
            .map(|row| {
                let org_pk =
                    serde_json::from_str::<UntrustedAnchorOrganizationPublicKey>(&row.org_pk_json)?;
                let org_pk = anchor_org_pk(&org_pk, now)?;

                let key_pair = serde_json::from_str::<UntrustedJournalistProvisioningKeyPair>(
                    &row.key_pair_json,
                )?;
                let key_pair = key_pair.to_trusted(&org_pk, now)?;

                Ok(key_pair)
            })
            .collect::<anyhow::Result<_>>()?;

        Ok(key_pairs)
    }

    // CoverNode Provisioning Keys

    pub async fn insert_covernode_provisioning_key_pair(
        &self,
        org_pk: &AnchorOrganizationPublicKey,
        key_pair: &CoverNodeProvisioningKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;

        let org_pk_hex = org_pk.public_key_hex();

        let org_pk_id = sqlx::query_scalar!(
            r#"
                SELECT
                    id AS "id: i64"
                FROM organization_public_keys
                WHERE json_extract(pk_json, '$.key') == ?1
            "#,
            org_pk_hex
        )
        .fetch_one(&mut *tx)
        .await?;

        let key_pair = key_pair.to_untrusted();
        let key_pair = serde_json::to_string(&key_pair)?;

        sqlx::query!(
            r#"
                INSERT INTO covernode_provisioning_key_pairs
                (organization_pk_id, key_pair_json, added_at)
                VALUES
                (?1, ?2, ?3)
                ON CONFLICT DO NOTHING
            "#,
            org_pk_id,
            key_pair,
            now,
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn select_covernode_provisioning_key_pairs(
        &self,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Vec<CoverNodeProvisioningKeyPair>> {
        let mut conn = self.pool.acquire().await?;

        let rows = sqlx::query!(
            r#"
                SELECT
                    key_pair_json AS "key_pair_json: String",
                    pk_json       AS "org_pk_json: String"
                FROM covernode_provisioning_key_pairs cp
                INNER JOIN organization_public_keys o
                    ON o.id = cp.organization_pk_id
            "#,
        )
        .fetch_all(&mut *conn)
        .await?;

        let key_pairs = rows
            .into_iter()
            .map(|row| {
                let org_pk =
                    serde_json::from_str::<UntrustedAnchorOrganizationPublicKey>(&row.org_pk_json)?;
                let org_pk = anchor_org_pk(&org_pk, now)?;

                let key_pair = serde_json::from_str::<UntrustedCoverNodeProvisioningKeyPair>(
                    &row.key_pair_json,
                )?;
                let key_pair = key_pair.to_trusted(&org_pk, now)?;

                Ok(key_pair)
            })
            .collect::<anyhow::Result<_>>()?;

        Ok(key_pairs)
    }
}
