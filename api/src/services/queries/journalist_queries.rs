use chrono::{DateTime, Utc};
use common::{
    api::{
        forms::PostJournalistBody,
        models::{
            journalist_id::JournalistIdentity,
            journalist_id_and_id_pk_rotation_form::JournalistIdAndPublicKeyRotationForm,
        },
    },
    client::{JournalistProfile, JournalistStatus},
    crypto::keys::{public_key::PublicKey, signing::traits::PublicSigningKey},
    epoch::Epoch,
    identity_api::{
        forms::post_rotate_journalist_id::RotateJournalistIdPublicKeyForm,
        models::UntrustedJournalistIdPublicKeyWithEpoch,
    },
    protocol::keys::{
        JournalistIdPublicKey, JournalistMessagingPublicKey, JournalistProvisioningPublicKey,
        OrganizationPublicKey, UntrustedUnregisteredJournalistIdPublicKey,
    },
};
use serde_json::Value;
use sqlx::PgPool;

use crate::{constants::MAX_NON_DESK_JOURNALIST_DESCRIPTION_LEN, error::AppError};

#[derive(Clone)]
pub struct JournalistQueries {
    pool: PgPool,
}

impl JournalistQueries {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// The main get journalists function, used by all clients to get the list of journalists
    /// they can reach. It is important in the client side to fetch this data periodically
    /// to avoid signaling to an adversary that they are using CoverDrop.
    ///
    /// Expired keys will be returned for 7 days after they're out of date in case a user checks
    /// for messages after being offline for a week.
    pub async fn journalist_profiles(&self) -> anyhow::Result<Vec<JournalistProfile>> {
        let mut connection = self.pool.acquire().await?;

        let journalists = sqlx::query!(
            r#"
            SELECT
                jp.id           AS "id!: JournalistIdentity",
                jp.display_name AS "display_name!: String",
                jp.sort_name    AS "sort_name!: String",
                jp.description  AS "description!: String",
                jp.is_desk      AS "is_desk!: bool",
                js.status       AS "status!: JournalistStatus"
            FROM journalist_profiles jp
            JOIN journalist_statuses js
                ON jp.status_id = js.id
            WHERE js.status != 'HIDDEN_FROM_RESPONSE'
            "#,
        )
        .map(|row| {
            JournalistProfile::new(
                row.id,
                row.display_name,
                row.sort_name,
                row.description,
                row.is_desk,
                row.status,
            )
        })
        .fetch_all(&mut *connection)
        .await?;

        Ok(journalists)
    }

    pub async fn insert_journalist_profile(
        &self,
        body: PostJournalistBody,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let mut connection = self.pool.acquire().await?;

        sqlx::query!(
            r#"
            INSERT INTO journalist_profiles (
                id,
                display_name,
                sort_name,
                description,
                is_desk,
                added_at,
                status_id
            )
            VALUES ($1, $2, $3, $4, $5, $6,
                (SELECT id FROM journalist_statuses WHERE status = $7)
            )
            ON CONFLICT (id) DO UPDATE SET
                display_name = EXCLUDED.display_name,
                sort_name    = EXCLUDED.sort_name,
                description  = EXCLUDED.description,
                is_desk      = EXCLUDED.is_desk,
                status_id    = EXCLUDED.status_id
            "#,
            &body.id,
            &body.display_name,
            &body.sort_name,
            &body.description,
            body.is_desk,
            now,
            body.status.to_string(),
        )
        .execute(&mut *connection)
        .await?;

        Ok(())
    }

    /// Get the epoch assigned to a journalist ID public key
    /// This function makes no guarantees about the key being
    /// valid, not expired, etc. It simply just returns the epoch
    /// that the key has been assigned.
    pub async fn get_journalist_id_pk_with_epoch_from_ed25519_pk(
        &self,
        candidate_id_pk_hex: &str,
    ) -> anyhow::Result<Option<UntrustedJournalistIdPublicKeyWithEpoch>> {
        let mut connection = self.pool.acquire().await?;

        let pk_with_epoch = sqlx::query!(
            r#"
            SELECT
                journalist_id_pks.epoch   AS "epoch: Epoch",
                journalist_id_pks.pk_json AS "pk_json: Value"
            FROM journalist_id_pks
            WHERE journalist_id_pks.pk_json #>>'{key}' = $1
            "#,
            candidate_id_pk_hex
        )
        .fetch_optional(&mut *connection)
        .await?
        .map(|row| {
            serde_json::from_value(row.pk_json).map(|key| UntrustedJournalistIdPublicKeyWithEpoch {
                epoch: row.epoch,
                key,
            })
        })
        .transpose()?;

        Ok(pk_with_epoch)
    }

    pub async fn insert_journalist_provisioning_pk(
        &self,
        provisioning_pk: &JournalistProvisioningPublicKey,
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
                INSERT INTO journalist_provisioning_pks (org_pk_id, added_at, not_valid_after, pk_json)
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

    pub async fn latest_provisioning_pk_added_at(&self) -> anyhow::Result<Option<DateTime<Utc>>> {
        let mut conn = self.pool.acquire().await?;

        let row = sqlx::query!(
            r#"
                SELECT MAX(added_at) AS "added_at: DateTime<Utc>"
                FROM journalist_provisioning_pks
            "#,
        )
        .fetch_one(&mut *conn)
        .await?;

        Ok(row.added_at)
    }

    pub async fn insert_journalist_id_pk(
        &self,
        journalist_id: &JournalistIdentity,
        id_pk: &JournalistIdPublicKey,
        from_queue: bool,
        signing_pk: &JournalistProvisioningPublicKey,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Epoch> {
        let mut tx = self.pool.begin().await?;

        let id_pk_bytes = id_pk.as_bytes();

        if from_queue {
            // Check if the form in the queue matches the identity-api provided
            // key. Note that we have to force not-null since SQLx can't prove
            // that EXISTS will always return true or false, even though it will.
            let new_pk_matches_queued_pk: bool = sqlx::query_scalar!(
                r#"
                    SELECT EXISTS (
                        SELECT 1
                        FROM journalist_id_pk_rotation_queue
                        WHERE journalist_id = $1
                            AND decode(
                                    (convert_from(
                                        decode(form_json #>>'{body}', 'base64'),
                                        'utf8'
                                    )::jsonb #>>'{new_pk,key}'),
                                    'hex'
                                ) = $2
                    ) AS "matches_queued!: bool"
                "#,
                journalist_id,
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

        // Using 2 CTEs we do an insert and a select for the provided journalist id key
        // this is required because 'RETUNING epoch' does not return anything if the key already exists
        let row = sqlx::query!(
            r#"
                WITH insert_query AS (
                    INSERT INTO journalist_id_pks (journalist_profile_id, provisioning_pk_id, added_at, not_valid_after, pk_json)
                        VALUES ($1, $2, $3, $4, $5)
                    ON CONFLICT DO NOTHING
                    RETURNING epoch AS "epoch: Epoch"
                ),
                select_query AS (
                    SELECT epoch AS "epoch: Epoch"
                    FROM journalist_id_pks
                    WHERE (pk_json->>'key') = ($5->>'key')
                )
                SELECT * FROM insert_query
                UNION
                SELECT * FROM select_query
            "#,
            &journalist_id,
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
                "Database did not get epoch value after inserting journalist id public key"
            );
        };

        if from_queue {
            sqlx::query!(
                r#"
                    DELETE FROM journalist_id_pk_rotation_queue
                    WHERE journalist_id = $1
                        AND decode(
                                (convert_from(
                                    decode(form_json #>>'{body}', 'base64'),
                                    'utf8'
                                )::jsonb #>>'{new_pk,key}'),
                                'hex'
                            ) = $2
                "#,
                journalist_id,
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
        journalist_id: &JournalistIdentity,
    ) -> anyhow::Result<Option<DateTime<Utc>>> {
        let mut conn = self.pool.acquire().await?;

        let row = sqlx::query!(
            r#"
                SELECT MAX(added_at) AS "added_at: DateTime<Utc>"
                FROM journalist_id_pks
                WHERE journalist_profile_id = $1
            "#,
            journalist_id
        )
        .fetch_one(&mut *conn)
        .await?;

        Ok(row.added_at)
    }

    pub async fn insert_journalist_msg_pk(
        &self,
        id: &JournalistIdentity,
        msg_pk: JournalistMessagingPublicKey,
        signing_pk: &JournalistIdPublicKey,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Epoch> {
        let mut tx = self.pool.begin().await?;

        let id_pk_id = sqlx::query!(
            r#"
                SELECT id AS "id_pk_id"
                FROM journalist_id_pks
                WHERE decode(pk_json #>>'{key}', 'hex') = $1
            "#,
            signing_pk.as_bytes()
        )
        .map(|row| row.id_pk_id)
        .fetch_one(&mut *tx)
        .await?;

        let msg_pk = msg_pk.to_untrusted();

        // As with the journalist_id_pk
        // Using 2 CTEs we do an insert and a select for the provided covernode msg key
        // this is required because 'RETUNING epoch' does not return anything if the key already exists
        let row = sqlx::query!(
            r#"
                WITH insert_query AS (
                    INSERT INTO journalist_msg_pks (journalist_profile_id, id_pk_id, added_at, not_valid_after, pk_json)
                        VALUES ($1, $2, $3, $4, $5)
                    ON CONFLICT DO NOTHING
                    RETURNING epoch "epoch: Epoch"
                ),
                select_query AS (
                    SELECT epoch AS "epoch: Epoch"
                    FROM journalist_msg_pks
                    WHERE (pk_json->>'key') = ($5->>'key')
                )
                SELECT * FROM insert_query
                UNION
                SELECT * FROM select_query
            "#,
            &id,
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
                "Database did not get epoch value after inserting journalist messaging public key"
            );
        };

        tx.commit().await?;

        Ok(epoch)
    }

    pub async fn latest_msg_pk_added_at(
        &self,
        journalist_id: &JournalistIdentity,
    ) -> anyhow::Result<Option<DateTime<Utc>>> {
        let mut conn = self.pool.acquire().await?;

        let row = sqlx::query!(
            r#"
                SELECT MAX(added_at) AS "added_at: DateTime<Utc>"
                FROM journalist_msg_pks
                WHERE journalist_profile_id = $1
            "#,
            journalist_id
        )
        .fetch_one(&mut *conn)
        .await?;

        Ok(row.added_at)
    }

    pub async fn update_journalist_profile(
        &self,
        journalist_id: JournalistIdentity,
        display_name: Option<String>,
        sort_name: Option<String>,
        is_desk: Option<bool>,
        description: Option<String>,
    ) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;

        let row = sqlx::query!(
            r#"
            UPDATE journalist_profiles SET
                display_name = COALESCE($1, display_name),
                sort_name    = COALESCE($2, sort_name),
                description  = COALESCE($3, description),
                is_desk      = COALESCE($4, is_desk)
            WHERE id = $5
            RETURNING
                is_desk,
                LENGTH(description) AS "description_length!: i32"
            "#,
            display_name,
            sort_name,
            description,
            is_desk,
            &journalist_id
        )
        .fetch_one(&mut *tx)
        .await?;

        // Performing validation here allows us to only use a single database query
        if !row.is_desk && row.description_length > MAX_NON_DESK_JOURNALIST_DESCRIPTION_LEN as i32 {
            tx.rollback().await?;
            Err(AppError::JournalistDescriptionTooLong)?
        } else {
            tx.commit().await?;
            Ok(())
        }
    }

    pub async fn update_journalist_status(
        &self,
        journalist_id: JournalistIdentity,
        status: JournalistStatus,
    ) -> anyhow::Result<()> {
        let mut connection = self.pool.acquire().await?;

        sqlx::query!(
            r#"
            UPDATE journalist_profiles SET
                status_id = (
                    SELECT id FROM journalist_statuses
                    WHERE status = $1
                )
            WHERE id = $2
            "#,
            status.as_ref(),
            &journalist_id
        )
        .execute(&mut *connection)
        .await?;

        Ok(())
    }

    pub async fn delete_journalist(&self, id: &JournalistIdentity) -> anyhow::Result<()> {
        let mut connection = self.pool.acquire().await?;

        sqlx::query!("DELETE FROM journalist_profiles WHERE id = $1", &id)
            .execute(&mut *connection)
            .await?;

        Ok(())
    }

    pub async fn insert_journalist_id_pk_rotation_form(
        &self,
        journalist_id: &JournalistIdentity,
        form: &RotateJournalistIdPublicKeyForm,
        // The extracted PK from within the rotate key form, used to check that the
        // key has not already been published
        new_pk: &UntrustedUnregisteredJournalistIdPublicKey,
    ) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;

        let form_new_pk_bytes = new_pk.key.as_bytes();

        let new_pk_already_published: bool = sqlx::query_scalar!(
            r#"
                SELECT EXISTS (
                    SELECT 1
                    FROM journalist_id_pks
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
                INSERT INTO journalist_id_pk_rotation_queue (journalist_id, form_json)
                    VALUES ($1, $2)
                ON CONFLICT (journalist_id) DO UPDATE SET
                form_json = excluded.form_json
            "#,
            journalist_id,
            form_json,
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn select_journalist_id_pk_rotation_forms(
        &self,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Vec<JournalistIdAndPublicKeyRotationForm>> {
        let mut connection = self.pool.acquire().await?;

        let rows = sqlx::query!(
            r#"
                SELECT
                    journalist_id AS "journalist_id: JournalistIdentity",
                    form_json     AS "form_json: Value"
                FROM journalist_id_pk_rotation_queue
                WHERE (form_json->>'not_valid_after')::TIMESTAMPTZ > $1
            "#,
            now
        )
        .fetch_all(&mut *connection)
        .await?;

        rows.into_iter()
            .map(|row| {
                let form = serde_json::from_value(row.form_json)?;
                Ok(JournalistIdAndPublicKeyRotationForm::new(
                    row.journalist_id,
                    form,
                ))
            })
            .collect::<anyhow::Result<Vec<_>>>()
    }
}
