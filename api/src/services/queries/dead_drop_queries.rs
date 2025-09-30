use std::num::NonZeroU32;

use chrono::{DateTime, Utc};
use common::{
    api::models::{
        dead_drop_summary::DeadDropSummary,
        dead_drops::{
            DeadDropId, JournalistToUserDeadDropSignatureDataV2,
            SerializedJournalistToUserDeadDropMessages, SerializedUserToJournalistDeadDropMessages,
            UnpublishedJournalistToUserDeadDrop, UnpublishedUserToJournalistDeadDrop,
            UnverifiedJournalistToUserDeadDrop, UnverifiedUserToJournalistDeadDrop,
            UserToJournalistDeadDropSignatureDataV2,
        },
    },
    crypto::{Signable, Signature, Verified},
    epoch::Epoch,
};
use sqlx::PgPool;

use crate::error::AppError;

#[derive(Clone)]
pub struct DeadDropQueries {
    pool: PgPool,
}

impl DeadDropQueries {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_journalist_to_user_dead_drops(
        &self,
        ids_greater_than: DeadDropId,
        limit: NonZeroU32,
    ) -> Result<Vec<UnverifiedJournalistToUserDeadDrop>, AppError> {
        let mut connection = self.pool.acquire().await?;

        let dead_drops = sqlx::query_as!(
            UnverifiedJournalistToUserDeadDrop,
            r#"
            SELECT
                id,
                created_at AS "created_at: DateTime<Utc>",
                data       AS "data: SerializedJournalistToUserDeadDropMessages",
                signature  AS "signature: Signature<JournalistToUserDeadDropSignatureDataV2>"
            FROM user_dead_drops
            WHERE id > $1
            ORDER BY id ASC
            LIMIT $2
            "#,
            ids_greater_than,
            limit.get() as i64
        )
        .fetch_all(&mut *connection)
        .await?;

        Ok(dead_drops)
    }

    pub async fn get_user_to_journalist_dead_drops(
        &self,
        ids_greater_than: DeadDropId,
        limit: NonZeroU32,
    ) -> Result<Vec<UnverifiedUserToJournalistDeadDrop>, AppError> {
        let mut connection = self.pool.acquire().await?;

        let dead_drops = sqlx::query_as!(
            UnverifiedUserToJournalistDeadDrop,
            r#"
            SELECT
                id,
                created_at AS "created_at: DateTime<Utc>",
                data       AS "data: SerializedUserToJournalistDeadDropMessages",
                signature  AS "signature: Signature<UserToJournalistDeadDropSignatureDataV2>",
                epoch      AS "epoch: Epoch"
            FROM journalist_dead_drops
            WHERE id > $1
            ORDER BY id ASC
            LIMIT $2
            "#,
            ids_greater_than,
            limit.get() as i64
        )
        .fetch_all(&mut *connection)
        .await?;

        Ok(dead_drops)
    }

    pub async fn add_journalist_to_user_dead_drop(
        &self,
        message: Verified<UnpublishedJournalistToUserDeadDrop>,
        now: DateTime<Utc>,
    ) -> Result<DeadDropId, AppError> {
        let mut connection = self.pool.acquire().await?;

        let data = message.data.as_signable_bytes();
        let signature = message.signature.to_bytes();
        let created_at = message.created_at;

        let id = sqlx::query_scalar!(
            r#"
                INSERT INTO user_dead_drops (data, signature, created_at, published_at)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT DO NOTHING
                RETURNING id
            "#,
            data,
            &signature,
            created_at,
            now
        )
        .fetch_one(&mut *connection)
        .await?;

        Ok(id)
    }

    pub async fn add_user_to_journalist_dead_drop(
        &self,
        message: Verified<UnpublishedUserToJournalistDeadDrop>,
        now: DateTime<Utc>,
    ) -> Result<DeadDropId, AppError> {
        let mut connection = self.pool.acquire().await?;

        let data = message.data.as_bytes();
        let signature = message.signature.to_bytes();
        let created_at = message.created_at;
        let epoch = *message.epoch;

        let id = sqlx::query_scalar!(
            r#"
                INSERT INTO journalist_dead_drops (data, signature, created_at, epoch, published_at)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT DO NOTHING
                RETURNING id
            "#,
            data,
            &signature,
            created_at,
            epoch,
            now
        )
        .fetch_one(&mut *connection)
        .await?;

        Ok(id)
    }

    pub async fn delete_old_dead_drops(&self, now: DateTime<Utc>) -> Result<(), AppError> {
        let mut connection = self.pool.acquire().await?;

        let journalist_query = sqlx::query!(
            "DELETE FROM journalist_dead_drops WHERE created_at + INTERVAL '14 days' < $1",
            now,
        )
        .execute(&mut *connection)
        .await?;

        let user_query = sqlx::query!(
            "DELETE FROM user_dead_drops WHERE created_at + INTERVAL '14 days' < $1",
            now,
        )
        .execute(&mut *connection)
        .await?;

        tracing::info!(
            "Deleted {} journalist dead drops and {} user dead drops",
            journalist_query.rows_affected(),
            user_query.rows_affected()
        );

        Ok(())
    }

    pub async fn get_journalist_to_user_recent_dead_drop_summary(
        &self,
    ) -> Result<Vec<DeadDropSummary>, AppError> {
        let mut connection = self.pool.acquire().await?;

        let max_id = sqlx::query_as!(
            DeadDropSummary,
            r#"
                SELECT
                    id         AS "id: DeadDropId",
                    created_at AS "created_at: DateTime<Utc>"
                FROM user_dead_drops
                ORDER BY id DESC
                LIMIT 10
            "#
        )
        .fetch_all(&mut *connection)
        .await?;

        Ok(max_id)
    }

    pub async fn get_user_to_journalist_recent_dead_drop_summary(
        &self,
    ) -> Result<Vec<DeadDropSummary>, AppError> {
        let mut connection = self.pool.acquire().await?;

        let max_id = sqlx::query_as!(
            DeadDropSummary,
            r#"
                SELECT
                    id         AS "id: DeadDropId",
                    created_at AS "created_at: DateTime<Utc>"
                FROM journalist_dead_drops
                ORDER BY id DESC
                LIMIT 10
            "#
        )
        .fetch_all(&mut *connection)
        .await?;

        Ok(max_id)
    }
}
