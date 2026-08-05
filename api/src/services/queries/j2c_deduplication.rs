use chrono::{DateTime, Utc};
use common::api::models::message_id::MessageId;
use common::protocol::constants::COVERNODE_MSG_KEY_VALID_DURATION;
use common::time;
use sqlx::PgPool;

#[derive(Clone)]
pub struct J2cDeduplicationQueries {
    pool: PgPool,
}

impl J2cDeduplicationQueries {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Attempts to insert a deduplication ID. Returns `true` if the insert
    /// succeeded (new ID), or `false` if it already exists (duplicate).
    pub async fn insert_deduplication_id(
        &self,
        deduplication_id: MessageId,
    ) -> anyhow::Result<bool> {
        let mut connection = self.pool.acquire().await?;

        let result = sqlx::query!(
            r#"
            INSERT INTO j2c_deduplication_ids (deduplication_id, created_at)
            VALUES ($1, $2)
            ON CONFLICT (deduplication_id) DO NOTHING
            "#,
            deduplication_id as MessageId,
            time::now()
        )
        .execute(&mut *connection)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Deletes deduplication IDs older than COVERNODE_MSG_KEY_VALID_DURATION.
    pub async fn delete_old_deduplication_ids(&self, now: DateTime<Utc>) -> anyhow::Result<()> {
        let mut connection = self.pool.acquire().await?;
        let cutoff = now - COVERNODE_MSG_KEY_VALID_DURATION;

        let result = sqlx::query!(
            "DELETE FROM j2c_deduplication_ids WHERE created_at < $1",
            cutoff,
        )
        .execute(&mut *connection)
        .await?;

        tracing::info!(
            "Deleted {} old j2c deduplication IDs",
            result.rows_affected()
        );

        Ok(())
    }
}
