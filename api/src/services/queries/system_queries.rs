use common::api::{
    forms::PostSystemStatusEventBody,
    models::general::{StatusEvent, SystemStatus},
};

use chrono::{DateTime, Utc};
use sqlx::PgPool;

#[derive(Clone)]
pub struct SystemQueries {
    pool: PgPool,
}

impl SystemQueries {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_latest_status(&self) -> anyhow::Result<Option<StatusEvent>> {
        let mut connection = self.pool.acquire().await?;

        let status = sqlx::query_as!(
            StatusEvent,
            r#"
            SELECT
                status      AS "status: SystemStatus",
                description AS "description: String",
                timestamp   AS "timestamp: DateTime<Utc>"
            FROM system_status_events
            ORDER BY timestamp
            DESC
            LIMIT 1
            "#,
        )
        .fetch_optional(&mut *connection)
        .await?;

        Ok(status)
    }

    pub async fn insert_status_event(
        &self,
        body: &PostSystemStatusEventBody,
    ) -> anyhow::Result<()> {
        let mut connection = self.pool.acquire().await?;

        let StatusEvent {
            status,
            description,
            timestamp,
        } = &body.status;

        sqlx::query!(
            r#"
            INSERT INTO system_status_events (
                status,
                description,
                timestamp
            )
            VALUES ($1, $2, $3)
            "#,
            status.as_ref(),
            description,
            timestamp
        )
        .execute(&mut *connection)
        .await?;

        Ok(())
    }
}
