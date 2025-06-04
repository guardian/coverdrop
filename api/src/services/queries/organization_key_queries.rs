use chrono::{DateTime, Utc};
use common::protocol::keys::{
    anchor_org_pk, AnchorOrganizationPublicKey, UntrustedAnchorOrganizationPublicKey,
};
use sqlx::PgPool;

#[derive(Clone)]
pub struct OrganizationKeyQueries {
    pool: PgPool,
}

impl OrganizationKeyQueries {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn org_pks(
        &self,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Vec<AnchorOrganizationPublicKey>> {
        let mut conn = self.pool.acquire().await?;

        let org_pks = sqlx::query!("SELECT pk_json FROM organization_pks")
            .fetch_all(&mut *conn)
            .await?
            .into_iter()
            .flat_map(|row| {
                let org_pk =
                    serde_json::from_value::<UntrustedAnchorOrganizationPublicKey>(row.pk_json)?;

                anchor_org_pk(&org_pk, now)
            })
            .collect::<Vec<AnchorOrganizationPublicKey>>();

        Ok(org_pks)
    }

    /// Inserts and organization public key.
    ///
    /// Returns `Ok(true)` if a new key was added.
    pub async fn insert_org_pk(
        &self,
        anchor_org_pk: &AnchorOrganizationPublicKey,
        now: DateTime<Utc>,
    ) -> anyhow::Result<bool> {
        let mut tx = self.pool.begin().await?;

        let anchor_org_pk = anchor_org_pk.to_untrusted();

        let serialized_org_pk = serde_json::to_value(&anchor_org_pk)?;

        let exists_row = sqlx::query!(
            r#"
                SELECT
                    EXISTS(
                        SELECT *
                        FROM organization_pks
                        WHERE pk_json = $1
                    ) AS "org_pk_exists!: bool"
            "#,
            serialized_org_pk
        )
        .fetch_one(&mut *tx)
        .await?;

        let mut did_insert_org_pk = false;

        if !exists_row.org_pk_exists {
            sqlx::query!(
                r#"
                INSERT INTO organization_pks (added_at, pk_json)
                    VALUES ($1, $2) ON CONFLICT DO NOTHING
                "#,
                now,
                serialized_org_pk,
            )
            .execute(&mut *tx)
            .await?;

            did_insert_org_pk = true;
        }

        tx.commit().await?;

        Ok(did_insert_org_pk)
    }
}
