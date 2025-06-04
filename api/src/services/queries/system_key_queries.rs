use chrono::{DateTime, Utc};
use common::{
    crypto::keys::{signing::traits::PublicSigningKey, Ed25519PublicKey},
    protocol::keys::{anchor_org_pk, OrganizationPublicKey, UntrustedAnchorOrganizationPublicKey},
    system::keys::{verify_admin_pk, AdminPublicKey, UntrustedAdminPublicKey},
};
use serde_json::Value;
use sqlx::PgPool;

#[derive(Clone)]
pub struct SystemKeyQueries {
    pool: PgPool,
}

impl SystemKeyQueries {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert_admin_pk(
        &self,
        admin_pk: &AdminPublicKey,
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

        let admin_pk = admin_pk.to_untrusted();

        sqlx::query!(
            r#"
                INSERT INTO admin_pks (org_pk_id, not_valid_after, pk_json)
                    VALUES ($1, $2, $3)
            "#,
            org_pk_id,
            admin_pk.not_valid_after,
            serde_json::to_value(&admin_pk)?,
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn find_admin_pk_from_ed25519_pk(
        &self,
        candidate_pk: &Ed25519PublicKey,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Option<AdminPublicKey>> {
        let mut connection = self.pool.acquire().await?;

        sqlx::query!(
            r#"
            SELECT
                admin_pks.pk_json AS "id_pk: Value",
                organization_pks.pk_json  AS "org_pk: Value"
            FROM admin_pks
            LEFT JOIN organization_pks
                ON admin_pks.org_pk_id = organization_pks.id
            WHERE decode(admin_pks.pk_json #>>'{key}', 'hex') = $1
            "#,
            candidate_pk.as_bytes()
        )
        .fetch_optional(&mut *connection)
        .await?
        .map(|row| {
            let org_pk =
                serde_json::from_value::<UntrustedAnchorOrganizationPublicKey>(row.org_pk)?;
            let org_pk = anchor_org_pk(&org_pk, now)?.to_non_anchor();

            let admin_pk = serde_json::from_value::<UntrustedAdminPublicKey>(row.id_pk)?;
            let admin_pk = verify_admin_pk(&admin_pk, &org_pk, now)?;
            anyhow::Ok(admin_pk)
        })
        .transpose()
    }
}
