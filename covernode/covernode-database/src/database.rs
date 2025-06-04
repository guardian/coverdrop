use std::path::Path;

use anyhow::Error;
use chrono::{DateTime, Utc};
use common::{
    api::forms::PostCoverNodeIdPublicKeyForm,
    argon2_sqlcipher::Argon2SqlCipher,
    epoch::Epoch,
    protocol::keys::{
        CoverNodeIdKeyPair, CoverNodeMessagingKeyPair, UnregisteredCoverNodeIdKeyPair,
        UntrustedCoverNodeIdKeyPair, UntrustedCoverNodeIdKeyPairWithEpoch,
        UntrustedCoverNodeMessagingKeyPair, UntrustedCoverNodeMessagingKeyPairWithEpoch,
    },
};

use crate::{
    UntrustedCandidateCoverNodeIdKeyPairWithCreatedAt,
    UntrustedCandidateCoverNodeMessagingKeyPairWithCreatedAt,
    UntrustedCoverNodeIdKeyPairWithCreatedAt,
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

    //
    // Setup bundle
    //

    pub async fn insert_setup_bundle(
        &self,
        form: &PostCoverNodeIdPublicKeyForm,
        key_pair: &CoverNodeIdKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let mut conn = self.pool.acquire().await?;

        let key_pair = key_pair.to_untrusted();
        let key_pair = serde_json::to_string(&key_pair)?;

        let form = serde_json::to_string(&form)?;

        sqlx::query!(
            r#"
                INSERT INTO setup_bundle
                (pk_upload_form_json, key_pair_json, created_at)
                VALUES
                (?1, ?2, ?3)
            "#,
            form,
            key_pair,
            now,
        )
        .execute(&mut *conn)
        .await?;

        Ok(())
    }

    pub async fn delete_setup_bundle(&self) -> anyhow::Result<()> {
        let mut conn = self.pool.begin().await?;

        let setup_bundle_delete_query = sqlx::query!(
            r#"
                DELETE FROM setup_bundle
            "#,
        )
        .execute(&mut *conn)
        .await?;

        let rows_affected = setup_bundle_delete_query.rows_affected();

        if rows_affected != 1 {
            conn.rollback().await?;
            anyhow::bail!(
                "Expected to delete 1 row from setup bundle, but deleted {} rows",
                rows_affected,
            )
        } else {
            tracing::info!("Deleted {} row from setup bundle", rows_affected);
            conn.commit().await?;
            Ok(())
        }
    }

    pub async fn select_setup_bundle(
        &self,
    ) -> anyhow::Result<
        Option<(
            PostCoverNodeIdPublicKeyForm,
            UntrustedCoverNodeIdKeyPairWithCreatedAt,
        )>,
    > {
        let mut conn = self.pool.acquire().await?;

        sqlx::query!(
            r#"
                SELECT
                    pk_upload_form_json AS "pk_upload_form_json: String",
                    key_pair_json AS "key_pair_json: String",
                    created_at AS "created_at: DateTime<Utc>"
                FROM setup_bundle
            "#,
        )
        .fetch_optional(&mut *conn)
        .await?
        .map(|row| {
            let pk_upload_form_json = serde_json::from_str(&row.pk_upload_form_json)?;
            let key_pair = serde_json::from_str(&row.key_pair_json)?;
            let created_at = row.created_at;

            let key_pair_with_created_at =
                UntrustedCoverNodeIdKeyPairWithCreatedAt::new(key_pair, created_at);

            Ok((pk_upload_form_json, key_pair_with_created_at))
        })
        .transpose()
    }

    //
    // Candidate Keys
    //
    // Keys which the CoverNode has generated but have yet to be assigned
    // in the system key hierarchy.
    //

    pub async fn select_candidate_id_key_pair(
        &self,
    ) -> anyhow::Result<Option<UntrustedCandidateCoverNodeIdKeyPairWithCreatedAt>> {
        let mut conn = self.pool.acquire().await?;

        sqlx::query!(
            r#"
                SELECT
                    key_pair_json AS "key_pair_json: String",
                    created_at AS "created_at: DateTime<Utc>"
                FROM covernode_id_key_pairs
                WHERE epoch IS NULL
            "#,
        )
        .fetch_optional(&mut *conn)
        .await?
        .map(|row| {
            let key_pair = serde_json::from_str(&row.key_pair_json)?;
            let created_at = row.created_at;

            Ok(UntrustedCandidateCoverNodeIdKeyPairWithCreatedAt::new(
                key_pair, created_at,
            ))
        })
        .transpose()
    }

    pub async fn select_candidate_msg_key_pair(
        &self,
    ) -> anyhow::Result<Option<UntrustedCandidateCoverNodeMessagingKeyPairWithCreatedAt>> {
        let mut conn = self.pool.acquire().await?;

        sqlx::query!(
            r#"
                SELECT
                    key_pair_json AS "key_pair_json: String",
                    created_at AS "created_at: DateTime<Utc>"
                FROM covernode_msg_key_pairs
                WHERE epoch IS NULL
            "#,
        )
        .fetch_optional(&mut *conn)
        .await?
        .map(|row| {
            let key_pair = serde_json::from_str(&row.key_pair_json)?;
            let created_at = row.created_at;

            Ok(UntrustedCandidateCoverNodeMessagingKeyPairWithCreatedAt::new(key_pair, created_at))
        })
        .transpose()
    }

    pub async fn insert_candidate_id_key_pair(
        &self,
        key_pair: &UnregisteredCoverNodeIdKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let mut conn = self.pool.acquire().await?;

        let key_pair = key_pair.to_untrusted();
        let key_pair = serde_json::to_string(&key_pair)?;

        sqlx::query!(
            r#"
                INSERT INTO covernode_id_key_pairs
                (key_pair_json, created_at)
                VALUES
                (?1, ?2)
            "#,
            key_pair,
            now,
        )
        .execute(&mut *conn)
        .await?;

        Ok(())
    }

    pub async fn insert_candidate_msg_key_pair(
        &self,
        key_pair: &CoverNodeMessagingKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let mut conn = self.pool.acquire().await?;

        let key_pair = key_pair.to_untrusted();
        let key_pair = serde_json::to_string(&key_pair)?;

        sqlx::query!(
            r#"
                INSERT INTO covernode_msg_key_pairs
                (key_pair_json, created_at)
                VALUES
                (?1, ?2)
            "#,
            key_pair,
            now,
        )
        .execute(&mut *conn)
        .await?;

        Ok(())
    }

    pub async fn update_candidate_id_key_pair_add_epoch(
        &self,
        key_pair: &CoverNodeIdKeyPair,
        epoch: Epoch,
    ) -> anyhow::Result<()> {
        let mut conn = self.pool.acquire().await?;

        let key_pair = key_pair.to_untrusted();
        let key_pair = serde_json::to_string(&key_pair)?;

        // here we are also replacing the unregistered (unsigned by provisioning key) key pair with a signed key pair
        // which was provided by the identity api
        sqlx::query!(
            r#"
                UPDATE covernode_id_key_pairs
                SET
                    epoch = ?1,
                    key_pair_json = ?2
                WHERE json_extract(key_pair_json, '$.secret_key') = json_extract(?2, '$.secret_key')
            "#,
            epoch,
            key_pair,
        )
        .execute(&mut *conn)
        .await?;

        Ok(())
    }

    pub async fn insert_id_key_pair_with_epoch(
        &self,
        key_pair: &CoverNodeIdKeyPair,
        epoch: Epoch,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let mut conn = self.pool.acquire().await?;

        let key_pair = key_pair.to_untrusted();
        let key_pair = serde_json::to_string(&key_pair)?;

        sqlx::query!(
            r#"
                INSERT INTO covernode_id_key_pairs
                (key_pair_json, epoch, created_at)
                VALUES
                (?1, ?2, ?3)
                ON CONFLICT DO NOTHING
            "#,
            key_pair,
            epoch,
            now
        )
        .execute(&mut *conn)
        .await?;

        Ok(())
    }

    pub async fn update_candidate_msg_key_pair_add_epoch(
        &self,
        // Does not need to be verified because we're just using this key to
        // look up the existing value in the database
        key_pair: &UntrustedCoverNodeMessagingKeyPair,
        epoch: Epoch,
    ) -> anyhow::Result<()> {
        let mut conn = self.pool.acquire().await?;

        let key_pair = serde_json::to_string(&key_pair)?;

        sqlx::query!(
            r#"
                UPDATE covernode_msg_key_pairs
                SET epoch = ?1
                WHERE json_extract(key_pair_json, '$.secret_key') = json_extract(?2, '$.secret_key')
            "#,
            epoch,
            key_pair,
        )
        .execute(&mut *conn)
        .await?;

        Ok(())
    }

    #[cfg(feature = "test-utils")]
    pub async fn insert_msg_key_pair_add_epoch(
        &self,
        // Does not need to be verified because we're just using this key to
        // look up the existing value in the database
        key_pair: &CoverNodeMessagingKeyPair,
        epoch: Epoch,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let mut conn = self.pool.acquire().await?;

        let key_pair = key_pair.to_untrusted();
        let key_pair = serde_json::to_string(&key_pair)?;

        sqlx::query!(
            r#"
                INSERT INTO covernode_msg_key_pairs
                (key_pair_json, epoch, created_at)
                VALUES
                (?1, ?2, ?3)
            "#,
            key_pair,
            epoch,
            now,
        )
        .execute(&mut *conn)
        .await?;

        Ok(())
    }

    //
    // Published keys (keys with an epoch)
    //

    pub async fn select_published_id_key_pairs(
        &self,
    ) -> anyhow::Result<Vec<UntrustedCoverNodeIdKeyPairWithEpoch>, Error> {
        let mut conn = self.pool.acquire().await?;

        let rows = sqlx::query!(
            r#"
                SELECT
                    key_pair_json AS "key_pair_json: String",
                    epoch AS "epoch: Epoch"
                FROM covernode_id_key_pairs
                WHERE epoch IS NOT NULL
            "#,
        )
        .fetch_all(&mut *conn)
        .await?;

        let published_key_pairs_with_epoch = rows
            .into_iter()
            .flat_map(|row| {
                let id_key_pair =
                    serde_json::from_str::<UntrustedCoverNodeIdKeyPair>(&row.key_pair_json)?;

                let res = row
                    .epoch
                    .map(|epoch| UntrustedCoverNodeIdKeyPairWithEpoch::new(id_key_pair, epoch));

                anyhow::Ok(res)
            })
            .flatten()
            .collect();

        Ok(published_key_pairs_with_epoch)
    }

    pub async fn select_published_msg_key_pairs(
        &self,
    ) -> anyhow::Result<Vec<UntrustedCoverNodeMessagingKeyPairWithEpoch>> {
        let mut conn = self.pool.acquire().await?;

        let rows = sqlx::query!(
            r#"
                SELECT
                    key_pair_json AS "key_pair_json: String",
                    epoch AS "epoch: Epoch"
                FROM covernode_msg_key_pairs
                WHERE epoch IS NOT NULL
            "#,
        )
        .fetch_all(&mut *conn)
        .await?;

        let published_key_pairs_with_epoch = rows
            .into_iter()
            .flat_map(|row| {
                let msg_key =
                    serde_json::from_str::<UntrustedCoverNodeMessagingKeyPair>(&row.key_pair_json)?;

                let res = row
                    .epoch
                    .map(|epoch| UntrustedCoverNodeMessagingKeyPairWithEpoch::new(msg_key, epoch));

                anyhow::Ok(res)
            })
            .flatten() // Flatten away results without an epoch, should never happen.
            .collect::<Vec<UntrustedCoverNodeMessagingKeyPairWithEpoch>>();

        Ok(published_key_pairs_with_epoch)
    }

    //
    // Deletion
    //

    pub async fn delete_expired_id_key_pairs(&self, now: DateTime<Utc>) -> anyhow::Result<()> {
        let mut conn = self.pool.acquire().await?;

        sqlx::query!(
            r#"
                DELETE FROM covernode_id_key_pairs
                WHERE json_extract(key_pair_json, '$.public_key.not_valid_after') < ?1
            "#,
            now
        )
        .execute(&mut *conn)
        .await?;

        Ok(())
    }

    pub async fn delete_expired_msg_key_pairs(&self, now: DateTime<Utc>) -> anyhow::Result<()> {
        let mut conn = self.pool.acquire().await?;

        sqlx::query!(
            r#"
                DELETE FROM covernode_msg_key_pairs
                WHERE json_extract(key_pair_json, '$.public_key.not_valid_after') < ?1
            "#,
            now
        )
        .execute(&mut *conn)
        .await?;

        Ok(())
    }
}
