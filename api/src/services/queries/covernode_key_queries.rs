use chrono::{DateTime, Utc};
use common::{
    api::models::covernode_id::CoverNodeIdentity,
    crypto::keys::signing::traits::PublicSigningKey,
    epoch::Epoch,
    protocol::keys::{
        CoverNodeIdPublicKey, CoverNodeMessagingPublicKey, CoverNodeProvisioningPublicKey,
        OrganizationPublicKey,
    },
};
use sqlx::PgPool;

#[derive(Clone)]
pub struct CoverNodeKeyQueries {
    pool: PgPool,
}

impl CoverNodeKeyQueries {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert_covernode_provisioning_pk(
        &self,
        provisioning_pk: &CoverNodeProvisioningPublicKey,
        signing_pk: &OrganizationPublicKey,
        now: DateTime<Utc>,
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

        let provisioning_pk = provisioning_pk.to_untrusted();

        sqlx::query!(
            r#"
                INSERT INTO covernode_provisioning_pks (org_pk_id, added_at, not_valid_after, pk_json)
                    VALUES ($1, $2, $3, $4)
            "#,
            org_pk_id,
            now,
            provisioning_pk.not_valid_after,
            serde_json::to_value(&provisioning_pk)?,
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn insert_covernode_id_pk(
        &self,
        covernode_id: &CoverNodeIdentity,
        id_pk: &CoverNodeIdPublicKey,
        signing_pk: &CoverNodeProvisioningPublicKey,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Epoch> {
        let mut tx = self.pool.begin().await?;

        let provisioning_pk_id = sqlx::query!(
            r#"
                SELECT id
                FROM covernode_provisioning_pks
                WHERE decode(pk_json #>>'{key}', 'hex') = $1
            "#,
            signing_pk.as_bytes()
        )
        .map(|row| row.id)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query!(
            r#"
                INSERT INTO covernodes (id, added_at)
                    VALUES ($1, $2)
                ON CONFLICT DO NOTHING
            "#,
            &covernode_id,
            now,
        )
        .execute(&mut *tx)
        .await?;

        let id_pk = id_pk.to_untrusted();

        // Using 2 CTEs we do an insert and a select for the provided covernode id key
        // this is required because 'RETUNING epoch' does not return anything if the key already exists
        let row = sqlx::query!(
            r#"
                WITH insert_query AS (
                    INSERT INTO covernode_id_pks (covernode_id, provisioning_pk_id, added_at, not_valid_after, pk_json)
                        VALUES ($1, $2, $3, $4, $5)
                    ON CONFLICT DO NOTHING
                    RETURNING epoch AS "epoch: Epoch"
                ),
                select_query AS (
                    SELECT epoch AS "epoch: Epoch"
                    FROM covernode_id_pks
                    WHERE (pk_json->>'key') = ($5->>'key')
                )
                SELECT * FROM insert_query
                UNION
                SELECT * FROM select_query
            "#,
            &covernode_id,
            provisioning_pk_id,
            now,
            id_pk.not_valid_after,
            serde_json::to_value(&id_pk)?,
        )
        .fetch_one(&mut *tx)
        .await?;

        let Some(epoch) = row.epoch else {
            // This should never happen but sqlx can't statically verify that epoch will always exist
            anyhow::bail!(
                "Database did not get epoch value after inserting covernode id public key"
            );
        };

        tx.commit().await?;

        Ok(epoch)
    }

    pub async fn latest_provisioning_pk_added_at(&self) -> anyhow::Result<Option<DateTime<Utc>>> {
        let mut conn = self.pool.acquire().await?;

        let row = sqlx::query!(
            r#"
                SELECT MAX(added_at) AS "added_at: DateTime<Utc>"
                FROM covernode_provisioning_pks
            "#
        )
        .fetch_one(&mut *conn)
        .await?;

        Ok(row.added_at)
    }

    pub async fn latest_id_pk_added_at(
        &self,
        covernode_id: &CoverNodeIdentity,
    ) -> anyhow::Result<Option<DateTime<Utc>>> {
        let mut conn = self.pool.acquire().await?;

        let row = sqlx::query!(
            r#"
                SELECT MAX(added_at) AS "added_at: DateTime<Utc>"
                FROM covernode_id_pks
                WHERE covernode_id = $1
            "#,
            covernode_id
        )
        .fetch_one(&mut *conn)
        .await?;

        Ok(row.added_at)
    }

    pub async fn insert_covernode_msg_pk(
        &self,
        covernode_id: &CoverNodeIdentity,
        msg_pk: &CoverNodeMessagingPublicKey,
        signing_pk: &CoverNodeIdPublicKey,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Epoch> {
        let mut tx = self.pool.begin().await?;

        let id_pk_id = sqlx::query!(
            r#"
                SELECT id 
                FROM covernode_id_pks
                WHERE decode(pk_json #>>'{key}', 'hex') = $1 AND covernode_id = $2
            "#,
            signing_pk.as_bytes(),
            &covernode_id
        )
        .map(|row| row.id)
        .fetch_one(&mut *tx)
        .await?;

        let msg_pk = msg_pk.to_untrusted();

        // As with the covernode_id_pk
        // Using 2 CTEs we do an insert and a select for the provided covernode msg key
        // this is required because 'RETUNING epoch' does not return anything if the key already exists
        let row = sqlx::query!(
            r#"
                WITH insert_query AS (
                    INSERT INTO covernode_msg_pks (covernode_id, id_pk_id, added_at, not_valid_after, pk_json)
                        VALUES ($1, $2, $3, $4, $5)
                    ON CONFLICT DO NOTHING
                    RETURNING epoch "epoch: Epoch"
                ),
                select_query AS (
                    SELECT epoch AS "epoch: Epoch"
                    FROM covernode_msg_pks
                    WHERE (pk_json->>'key') = ($5->>'key')
                )
                SELECT * FROM insert_query
                UNION
                SELECT * FROM select_query
            "#,
            &covernode_id,
            id_pk_id,
            now,
            msg_pk.not_valid_after,
            serde_json::to_value(&msg_pk)?,
        )
        .fetch_one(&mut *tx)
        .await?;

        let Some(epoch) = row.epoch else {
            // This should never happen but sqlx can't statically verify that epoch will always exist
            anyhow::bail!(
                "Database did not get epoch value after inserting covernode messaging public key"
            );
        };

        tx.commit().await?;

        Ok(epoch)
    }

    pub async fn latest_msg_pk_added_at(
        &self,
        covernode_id: &CoverNodeIdentity,
    ) -> anyhow::Result<Option<DateTime<Utc>>> {
        let mut conn = self.pool.acquire().await?;

        let row = sqlx::query!(
            r#"
                SELECT MAX(added_at) AS "added_at: DateTime<Utc>"
                FROM covernode_msg_pks
                WHERE covernode_id = $1
            "#,
            covernode_id
        )
        .fetch_one(&mut *conn)
        .await?;

        Ok(row.added_at)
    }
}
