use std::path::Path;

use anyhow::Error;
use chrono::{DateTime, Utc};
use common::{
    api::forms::PostCoverNodeIdPublicKeyForm,
    argon2_sqlcipher::Argon2SqlCipher,
    aws::kinesis::{
        client::StreamKind,
        models::checkpoint::{Checkpoints, CheckpointsJson, StoredCheckpoints},
    },
    epoch::Epoch,
    protocol::keys::{
        CoverNodeIdKeyPair, CoverNodeMessagingKeyPair, UnregisteredCoverNodeIdKeyPair,
        UntrustedCoverNodeIdKeyPair, UntrustedCoverNodeIdKeyPairWithEpoch,
        UntrustedCoverNodeMessagingKeyPair, UntrustedCoverNodeMessagingKeyPairWithEpoch,
    },
};

use crate::{
    MessageHash, MessageHashExpiry, MessageHashesWithExpiries,
    UntrustedCandidateCoverNodeIdKeyPairWithCreatedAt,
    UntrustedCandidateCoverNodeMessagingKeyPairWithCreatedAt,
    UntrustedCoverNodeIdKeyPairWithCreatedAt,
};
use sqlx::{SqliteConnection, SqlitePool};

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

async fn update_checkpoint(
    conn: &mut SqliteConnection,
    stream_kind: StreamKind,
    checkpoints_json: CheckpointsJson,
) -> anyhow::Result<()> {
    let json = checkpoints_json.as_str();

    let result = sqlx::query!(
        r#"
                UPDATE checkpoints
                SET checkpoints_json = ?1
                WHERE stream_kind = ?2
            "#,
        json,
        stream_kind,
    )
    .execute(&mut *conn)
    .await?;

    let rows_affected = result.rows_affected();
    if rows_affected != 1 {
        anyhow::bail!(
            "Expected to update 1 checkpoint row, but updated {} rows",
            rows_affected,
        );
    }

    Ok(())
}

async fn insert_seen_message_hashes(
    conn: &mut SqliteConnection,
    stream_kind: StreamKind,
    hashes: &[(MessageHash, MessageHashExpiry)],
) -> anyhow::Result<()> {
    for (hash, expires_at) in hashes {
        sqlx::query!(
            r#"
                INSERT OR IGNORE INTO seen_message_hashes
                (stream_kind, hash, expires_at)
                VALUES (?1, ?2, ?3)
            "#,
            stream_kind,
            hash,
            expires_at
        )
        .execute(&mut *conn)
        .await?;
    }

    Ok(())
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
                    epoch AS "epoch: Epoch",
                    created_at AS "created_at: DateTime<Utc>"
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

                let created_at = row.created_at;

                let res = row.epoch.map(|epoch| {
                    UntrustedCoverNodeIdKeyPairWithEpoch::new(id_key_pair, epoch, created_at)
                });

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
                    epoch AS "epoch: Epoch",
                    created_at AS "created_at: DateTime<Utc>"
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

                let created_at = row.created_at;

                let res = row.epoch.map(|epoch| {
                    UntrustedCoverNodeMessagingKeyPairWithEpoch::new(msg_key, epoch, created_at)
                });

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

    //
    // Checkpoints
    //

    pub async fn select_checkpoints(&self) -> anyhow::Result<StoredCheckpoints> {
        let mut conn = self.pool.acquire().await?;

        let rows = sqlx::query!(
            r#"
                SELECT
                    stream_kind AS "stream_kind: StreamKind",
                    checkpoints_json AS "checkpoints_json: String"
                FROM checkpoints
            "#,
        )
        .fetch_all(&mut *conn)
        .await?;

        let mut user_to_journalist_checkpoints = Checkpoints::new();
        let mut journalist_to_user_checkpoints = Checkpoints::new();

        for row in rows {
            let checkpoints: Checkpoints = serde_json::from_str(&row.checkpoints_json)?;
            match row.stream_kind {
                StreamKind::UserToJournalist => user_to_journalist_checkpoints = checkpoints,
                StreamKind::JournalistToUser => journalist_to_user_checkpoints = checkpoints,
            }
        }

        Ok(StoredCheckpoints {
            user_to_journalist_checkpoints,
            journalist_to_user_checkpoints,
        })
    }

    //
    // Message hashes
    //

    pub async fn select_seen_message_hashes(
        &self,
        stream_kind: StreamKind,
        now: DateTime<Utc>,
    ) -> anyhow::Result<MessageHashesWithExpiries> {
        let mut conn = self.pool.acquire().await?;

        let rows = sqlx::query!(
            r#"
                SELECT hash AS "hash: MessageHash",
                       expires_at AS "expires_at: MessageHashExpiry"
                FROM seen_message_hashes
                WHERE stream_kind = ?1 AND expires_at > ?2
            "#,
            stream_kind,
            now
        )
        .fetch_all(&mut *conn)
        .await?;

        let hashes = rows
            .into_iter()
            .map(|row| (row.hash, row.expires_at))
            .collect();

        Ok(hashes)
    }

    pub async fn delete_expired_seen_message_hashes(
        &self,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let mut conn = self.pool.acquire().await?;

        sqlx::query!(
            r#"
            DELETE
               FROM seen_message_hashes
               WHERE expires_at <= ?1
            "#,
            now
        )
        .execute(&mut *conn)
        .await?;

        Ok(())
    }

    pub async fn update_checkpoint_and_insert_seen_message_hashes(
        &self,
        stream_kind: StreamKind,
        checkpoints_json: CheckpointsJson,
        hashes: &[(MessageHash, MessageHashExpiry)],
    ) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;
        update_checkpoint(&mut tx, stream_kind, checkpoints_json).await?;
        insert_seen_message_hashes(&mut tx, stream_kind, hashes).await?;
        tx.commit().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use common::aws::kinesis::models::checkpoint::{Checkpoints, SequenceNumber};
    use common::time;

    async fn update_checkpoints_in_transaction(
        db: &Database,
        stream_kind: StreamKind,
        checkpoints_json: CheckpointsJson,
    ) -> anyhow::Result<()> {
        let mut tx = db.pool.begin().await.expect("Transaction to be created");
        update_checkpoint(&mut tx, stream_kind, checkpoints_json).await?;
        tx.commit().await.expect("Transaction to be commited");
        Ok(())
    }

    async fn insert_seen_message_hash_in_transaction(
        db: &Database,
        stream_kind: StreamKind,
        hashes: &[(MessageHash, MessageHashExpiry)],
    ) {
        let mut tx = db.pool.begin().await.expect("Transaction to be created");
        insert_seen_message_hashes(&mut tx, stream_kind, hashes)
            .await
            .expect("Hashes to be inserted");
        tx.commit().await.expect("Transaction to be commited");
    }

    #[tokio::test]
    async fn update_checkpoint_errors_when_row_missing() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path, "test-password").await.unwrap();

        // Delete the seeded rows so the update has nothing to match
        sqlx::query("DELETE FROM checkpoints")
            .execute(&db.pool)
            .await
            .unwrap();

        let checkpoints = Checkpoints::new();
        let json = CheckpointsJson::new(&checkpoints).unwrap();

        let result =
            update_checkpoints_in_transaction(&db, StreamKind::UserToJournalist, json).await;

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Expected to update 1 checkpoint row, but updated 0 rows"),
            "Unexpected error message: {err_msg}"
        );
    }

    #[tokio::test]
    async fn update_and_select_checkpoints_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path, "test-password").await.unwrap();

        let mut u2j_checkpoints = Checkpoints::new();
        u2j_checkpoints.insert("shard-abc".to_string(), SequenceNumber::from("99999"));

        // U2J checkpoint round trip
        let checkpoints_json = CheckpointsJson::new(&u2j_checkpoints).unwrap();
        update_checkpoints_in_transaction(&db, StreamKind::UserToJournalist, checkpoints_json)
            .await
            .expect("Update to complete");

        let stored_checkpoints = db.select_checkpoints().await.unwrap();
        assert_eq!(
            stored_checkpoints.user_to_journalist_checkpoints,
            u2j_checkpoints
        );
        // J2U should still be empty
        assert_eq!(
            stored_checkpoints.journalist_to_user_checkpoints,
            Checkpoints::new()
        );

        // J2U checkpoint round trip
        let mut j2u_checkpoints = Checkpoints::new();
        j2u_checkpoints.insert("shard-xyz".to_string(), SequenceNumber::from("88888"));
        let checkpoints_json = CheckpointsJson::new(&j2u_checkpoints).unwrap();
        update_checkpoints_in_transaction(&db, StreamKind::JournalistToUser, checkpoints_json)
            .await
            .expect("Update to complete");
        let stored_checkpoints = db.select_checkpoints().await.unwrap();
        assert_eq!(
            stored_checkpoints.journalist_to_user_checkpoints,
            j2u_checkpoints
        );
    }

    #[tokio::test]
    async fn insert_check_and_expire_old_message_hashes() {
        // Establish DB
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path, "test-password").await.unwrap();

        let now = time::now();
        let expired_messaging_key = MessageHashExpiry::new(now - Duration::weeks(2));
        let valid_messaging_key = MessageHashExpiry::new(now);
        let message_hash_zeros = MessageHash::new([0_u8; 32]);
        let message_hash_ones = MessageHash::new([1_u8; 32]);

        // Messages to be inserted (one expired key, one not expired)
        let hashes: MessageHashesWithExpiries = vec![
            (message_hash_zeros, valid_messaging_key),
            (message_hash_ones, expired_messaging_key),
        ];

        insert_seen_message_hash_in_transaction(&db, StreamKind::UserToJournalist, &hashes).await;

        let hashes = db
            .select_seen_message_hashes(StreamKind::UserToJournalist, now)
            .await
            .expect("Hashes to be selected");

        // Only valid hashes should be selected out
        assert_eq!(hashes, vec![(message_hash_zeros, valid_messaging_key)]);
        assert_eq!(hashes.len(), 1);

        // There should be no JournalistToUser messages
        let hashes = db
            .select_seen_message_hashes(StreamKind::JournalistToUser, now)
            .await
            .expect("Hashes to be selected");

        assert_eq!(hashes, vec![]);
        assert_eq!(hashes.len(), 0);

        // Two weeks elapse
        let now = now + Duration::weeks(2);

        db.delete_expired_seen_message_hashes(now)
            .await
            .expect("Hashes to be deleted");

        let hashes = db
            .select_seen_message_hashes(StreamKind::UserToJournalist, now)
            .await
            .expect("Hashes to be selected");

        // No message hashes should remain
        assert_eq!(hashes, vec![]);
        assert_eq!(hashes.len(), 0);
    }

    #[tokio::test]
    async fn insert_duplicate_hashes() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path, "test-password").await.unwrap();

        let now = time::now();
        let now_plus_one = MessageHashExpiry::new(now + Duration::minutes(1));
        let now_plus_five = MessageHashExpiry::new(now + Duration::minutes(5));
        let message_hash = MessageHash::new([0_u8; 32]);

        let hashes: MessageHashesWithExpiries = vec![(message_hash, now_plus_one)];

        insert_seen_message_hash_in_transaction(&db, StreamKind::UserToJournalist, &hashes).await;

        let hashes: MessageHashesWithExpiries = vec![(message_hash, now_plus_five)];

        insert_seen_message_hash_in_transaction(&db, StreamKind::UserToJournalist, &hashes).await;

        insert_seen_message_hash_in_transaction(&db, StreamKind::UserToJournalist, &hashes).await;

        let hashes = db
            .select_seen_message_hashes(StreamKind::UserToJournalist, now)
            .await
            .expect("Hashes to be selected");

        assert_eq!(vec![(message_hash, now_plus_one)], hashes);
        assert_eq!(hashes.len(), 1);
    }

    #[tokio::test]
    async fn update_checkpoint_and_insert_seen_message_hashes_writes_both() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path, "test-password").await.unwrap();

        let now = time::now();
        let now_plus_five = MessageHashExpiry::new(now + Duration::minutes(5));

        let mut u2j_checkpoints = Checkpoints::new();
        u2j_checkpoints.insert("shard-abc".to_string(), SequenceNumber::from("99999"));
        let checkpoints_json = CheckpointsJson::new(&u2j_checkpoints).unwrap();

        let message_hash_zeros = MessageHash::new([0_u8; 32]);
        let message_hash_ones = MessageHash::new([1_u8; 32]);
        let hashes: MessageHashesWithExpiries = vec![
            (message_hash_zeros, now_plus_five),
            (message_hash_ones, now_plus_five),
        ];

        db.update_checkpoint_and_insert_seen_message_hashes(
            StreamKind::UserToJournalist,
            checkpoints_json,
            &hashes,
        )
        .await
        .expect("Checkpoints and hashes to be written");

        let stored_checkpoints = db.select_checkpoints().await.unwrap();
        assert_eq!(
            stored_checkpoints.user_to_journalist_checkpoints,
            u2j_checkpoints
        );

        let stored_hashes = db
            .select_seen_message_hashes(StreamKind::UserToJournalist, now)
            .await
            .expect("Hashes to be selected");

        assert_eq!(
            stored_hashes,
            vec![
                (message_hash_zeros, now_plus_five),
                (message_hash_ones, now_plus_five),
            ]
        );

        // The other stream must be untouched
        assert_eq!(
            stored_checkpoints.journalist_to_user_checkpoints,
            Checkpoints::new()
        );
        assert_eq!(
            db.select_seen_message_hashes(StreamKind::JournalistToUser, now)
                .await
                .expect("Hashes to be selected"),
            vec![]
        );
    }
}
