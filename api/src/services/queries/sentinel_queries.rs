use chrono::{DateTime, Utc};
use common::{
    api::models::{
        sentinel_id::SentinelIdentity,
        sentinel_id_and_id_pk_rotation_form::SentinelIdAndPublicKeyRotationForm,
    },
    client::SentinelProfile,
    crypto::keys::{public_key::PublicKey, signing::traits::PublicSigningKey},
    epoch::Epoch,
    identity_api::{
        forms::post_rotate_sentinel_id::RotateSentinelIdPublicKeyForm,
        models::UntrustedSentinelIdPublicKeyWithEpoch,
    },
    protocol::keys::{
        JournalistProvisioningPublicKey, SentinelIdPublicKey,
        UntrustedUnregisteredSentinelIdPublicKey,
    },
};
use serde_json::Value;
use sqlx::PgPool;

#[derive(Clone)]
pub struct SentinelQueries {
    pool: PgPool,
}

impl SentinelQueries {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn sentinel_profiles(&self) -> anyhow::Result<Vec<SentinelProfile>> {
        let mut connection = self.pool.acquire().await?;

        let sentinel_profiles = sqlx::query!(
            r#"
            SELECT
                id           AS "id!: SentinelIdentity",
                display_name AS "display_name!: String"
            FROM sentinel_profiles
            "#,
        )
        .map(|row| SentinelProfile::new(row.id, row.display_name))
        .fetch_all(&mut *connection)
        .await?;

        Ok(sentinel_profiles)
    }

    pub async fn insert_sentinel_profile(
        &self,
        id: SentinelIdentity,
        display_name: String,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let mut connection = self.pool.acquire().await?;

        sqlx::query!(
            r#"
            INSERT INTO sentinel_profiles (id, display_name, added_at)
            VALUES ($1, $2, $3)
            ON CONFLICT (id) DO UPDATE SET
                display_name = EXCLUDED.display_name
            "#,
            &id,
            &display_name,
            now,
        )
        .execute(&mut *connection)
        .await?;

        Ok(())
    }

    pub async fn insert_sentinel_id_pk_rotation_form(
        &self,
        sentinel_id: &SentinelIdentity,
        form: &RotateSentinelIdPublicKeyForm,
        new_pk: &UntrustedUnregisteredSentinelIdPublicKey,
    ) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;

        let form_new_pk_bytes = new_pk.key.as_bytes();

        let new_pk_already_published: bool = sqlx::query_scalar!(
            r#"
                SELECT EXISTS (
                    SELECT 1
                    FROM sentinel_id_pks
                    WHERE decode(pk_json #>>'{key}', 'hex') = $1
                ) AS "matches_queued!: bool"
            "#,
            form_new_pk_bytes,
        )
        .fetch_one(&mut *tx)
        .await?;

        if new_pk_already_published {
            anyhow::bail!(
                "New public key '{:?}' from rotation request has already been published",
                new_pk.public_key_hex()
            );
        }

        let form_json = serde_json::to_value(form)?;

        sqlx::query!(
            r#"
                INSERT INTO sentinel_id_pk_rotation_queue (sentinel_id, form_json)
                    VALUES ($1, $2)
                ON CONFLICT (sentinel_id) DO UPDATE SET
                form_json = excluded.form_json
            "#,
            sentinel_id,
            form_json,
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn select_sentinel_id_pk_rotation_forms(
        &self,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Vec<SentinelIdAndPublicKeyRotationForm>> {
        let mut connection = self.pool.acquire().await?;

        let rows = sqlx::query!(
            r#"
                SELECT
                    sentinel_id AS "sentinel_id: SentinelIdentity",
                    form_json   AS "form_json: Value"
                FROM sentinel_id_pk_rotation_queue
                WHERE (form_json->>'not_valid_after')::TIMESTAMPTZ > $1
            "#,
            now
        )
        .fetch_all(&mut *connection)
        .await?;

        rows.into_iter()
            .map(|row| {
                let form = serde_json::from_value(row.form_json)?;
                Ok(SentinelIdAndPublicKeyRotationForm::new(
                    row.sentinel_id,
                    form,
                ))
            })
            .collect::<anyhow::Result<Vec<_>>>()
    }

    pub async fn insert_sentinel_id_pk(
        &self,
        sentinel_id: &SentinelIdentity,
        id_pk: &SentinelIdPublicKey,
        from_queue: bool,
        signing_pk: &JournalistProvisioningPublicKey,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Epoch> {
        let mut tx = self.pool.begin().await?;

        let id_pk_bytes = id_pk.as_bytes();

        if from_queue {
            let new_pk_matches_queued_pk: bool = sqlx::query_scalar!(
                r#"
                    SELECT EXISTS (
                        SELECT 1
                        FROM sentinel_id_pk_rotation_queue
                        WHERE sentinel_id = $1
                            AND decode(
                                    (convert_from(
                                        decode(form_json #>>'{body}', 'base64'),
                                        'utf8'
                                    )::jsonb #>>'{new_pk,key}'),
                                    'hex'
                                ) = $2
                    ) AS "matches_queued!: bool"
                "#,
                sentinel_id,
                id_pk_bytes,
            )
            .fetch_one(&mut *tx)
            .await?;

            if !new_pk_matches_queued_pk {
                anyhow::bail!(
                    "Newly submitted public key does not match the version in the form queue"
                );
            }
        }

        let provisioning_pk_id = sqlx::query!(
            r#"
                SELECT id AS "provisioning_pk_id"
                FROM journalist_provisioning_pks
                WHERE decode(pk_json #>>'{key}', 'hex') = $1
            "#,
            signing_pk.as_bytes()
        )
        .map(|row| row.provisioning_pk_id)
        .fetch_one(&mut *tx)
        .await?;

        let id_pk = id_pk.to_untrusted();

        let row = sqlx::query!(
            r#"
                WITH insert_query AS (
                    INSERT INTO sentinel_id_pks (sentinel_id, provisioning_pk_id, added_at, not_valid_after, pk_json)
                        VALUES ($1, $2, $3, $4, $5)
                    ON CONFLICT DO NOTHING
                    RETURNING epoch AS "epoch: Epoch"
                ),
                select_query AS (
                    SELECT epoch AS "epoch: Epoch"
                    FROM sentinel_id_pks
                    WHERE (pk_json->>'key') = ($5->>'key')
                )
                SELECT * FROM insert_query
                UNION
                SELECT * FROM select_query
            "#,
            sentinel_id,
            provisioning_pk_id,
            now,
            id_pk.not_valid_after,
            serde_json::to_value(&id_pk)?,
        )
        .fetch_one(&mut *tx)
        .await?;

        let Some(epoch) = row.epoch else {
            anyhow::bail!(
                "Database did not get epoch value after inserting sentinel id public key"
            );
        };

        if from_queue {
            sqlx::query!(
                r#"
                    DELETE FROM sentinel_id_pk_rotation_queue
                    WHERE sentinel_id = $1
                        AND decode(
                                (convert_from(
                                    decode(form_json #>>'{body}', 'base64'),
                                    'utf8'
                                )::jsonb #>>'{new_pk,key}'),
                                'hex'
                            ) = $2
                "#,
                sentinel_id,
                id_pk_bytes
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(epoch)
    }

    pub async fn latest_id_pk_added_at(
        &self,
        sentinel_id: &SentinelIdentity,
    ) -> anyhow::Result<Option<DateTime<Utc>>> {
        let mut conn = self.pool.acquire().await?;

        let row = sqlx::query!(
            r#"
                SELECT MAX(added_at) AS "added_at: DateTime<Utc>"
                FROM sentinel_id_pks
                WHERE sentinel_id = $1
            "#,
            sentinel_id
        )
        .fetch_one(&mut *conn)
        .await?;

        Ok(row.added_at)
    }

    /// Get the epoch assigned to a sentinel ID public key
    /// This function makes no guarantees about the key being
    /// valid, not expired, etc. It simply returns the epoch
    /// that the key has been assigned.
    pub async fn get_sentinel_id_pk_with_epoch_from_ed25519_pk(
        &self,
        candidate_id_pk_hex: &str,
    ) -> anyhow::Result<Option<UntrustedSentinelIdPublicKeyWithEpoch>> {
        let mut connection = self.pool.acquire().await?;

        let pk_with_epoch = sqlx::query!(
            r#"
            SELECT
                sentinel_id_pks.epoch   AS "epoch: Epoch",
                sentinel_id_pks.pk_json AS "pk_json: Value"
            FROM sentinel_id_pks
            WHERE sentinel_id_pks.pk_json #>>'{key}' = $1
            "#,
            candidate_id_pk_hex
        )
        .fetch_optional(&mut *connection)
        .await?
        .map(|row| {
            serde_json::from_value(row.pk_json).map(|key| UntrustedSentinelIdPublicKeyWithEpoch {
                epoch: row.epoch,
                key,
            })
        })
        .transpose()?;

        Ok(pk_with_epoch)
    }
}
