use chrono;
use common::api::models::journalist_id::JournalistIdentity;
use common::client::mailbox::user_mailbox::MAX_MAILBOX_MESSAGES;
use common::crypto::keys::public_key::PublicKey;
use common::protocol::constants::JOURNALIST_MSG_KEY_VALID_DURATION;
use common::protocol::keys::{UntrustedUserKeyPair, UserKeyPair, UserPublicKey};
use reqwest::Url;
use serde_json::Value;
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    ConnectOptions, PgPool,
};

use crate::model::{AllReceivedJournalistToUserMessages, User};

#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(db_url: &str) -> anyhow::Result<Database> {
        let url = Url::parse(db_url).expect("Parse db url");

        // We disable statement logging so no connection secrets are sent to logs
        let connect_options = PgConnectOptions::from_url(&url)?.disable_statement_logging();

        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect_with(connect_options)
            .await?;

        sqlx::migrate!().run(&pool).await?;

        let db = Database { pool };

        Ok(db)
    }

    //
    // Users and journalists
    //

    pub async fn insert_user(&self, key_pair: UserKeyPair) -> anyhow::Result<()> {
        let mut connection = self.pool.acquire().await?;
        let key_pair = serde_json::to_value(key_pair.to_untrusted())?;

        sqlx::query!(
            r#"
                INSERT INTO users (
                    key_pair_json
                )
                VALUES ($1)
            "#,
            key_pair
        )
        .execute(&mut *connection)
        .await?;

        Ok(())
    }

    pub async fn get_users(&self, num_users: u16) -> anyhow::Result<Vec<User>> {
        let mut connection = self.pool.acquire().await?;
        let users = sqlx::query!(
            r#"
                SELECT
                    id AS "user_id: i32",
                    key_pair_json AS "key_pair_json: Value"
                FROM users
                ORDER BY id ASC
                LIMIT $1
            "#,
            num_users as i32
        )
        .fetch_all(&mut *connection)
        .await?
        .into_iter()
        .map(|row| {
            let key_pair = serde_json::from_value::<UntrustedUserKeyPair>(row.key_pair_json)?;
            let key_pair = key_pair.to_trusted();

            anyhow::Ok(User {
                user_id: row.user_id,
                key_pair,
            })
        })
        .collect::<anyhow::Result<_>>()?;

        Ok(users)
    }

    pub async fn insert_journalist(&self, id: &JournalistIdentity) -> anyhow::Result<()> {
        let mut connection = self.pool.acquire().await?;
        sqlx::query!(
            r#"
                INSERT INTO journalists (id)
                VALUES ($1)
                ON CONFLICT DO NOTHING
            "#,
            id,
        )
        .execute(&mut *connection)
        .await?;

        Ok(())
    }

    pub async fn get_journalists(&self) -> anyhow::Result<Vec<JournalistIdentity>> {
        let mut connection = self.pool.acquire().await?;
        let journalists = sqlx::query_scalar!(
            r#"SELECT id AS "journalist_id: JournalistIdentity" FROM journalists"#
        )
        .fetch_all(&mut *connection)
        .await?;

        Ok(journalists)
    }

    //
    // Messages
    //

    pub async fn insert_user_to_journalist_message(
        &self,
        user_id: i32,
        journalist_id: &JournalistIdentity,
        message: &str,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let mut connection = self.pool.acquire().await?;

        sqlx::query!(
            r#"
                INSERT INTO user_to_journalist_messages (
                    user_id,
                    journalist_id,
                    sent_at,
                    message
                ) VALUES
                ($1, $2, $3, $4)
            "#,
            user_id,
            &journalist_id,
            now,
            message
        )
        .execute(&mut *connection)
        .await?;

        Ok(())
    }

    pub async fn insert_j2u_message(
        &self,
        journalist_id: &JournalistIdentity,
        user_pk: &UserPublicKey,
        message: &str,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let mut conn = self.pool.acquire().await?;

        let user_pk_hex = user_pk.public_key_hex();

        sqlx::query!(
            r#"
                WITH user_id AS (
                    SELECT id
                    FROM users
                    WHERE (key_pair_json -> 'public_key' ->> 'key') = $1
                )
                INSERT INTO journalist_to_user_messages (
                    user_id,
                    journalist_id,
                    sent_at,
                    message
                )
                SELECT user_id.id, $2, $3, $4
                FROM user_id
            "#,
            user_pk_hex,
            &journalist_id,
            now,
            message
        )
        .execute(&mut *conn)
        .await?;

        Ok(())
    }

    pub async fn get_all_received_j2u_messages(
        &self,
    ) -> anyhow::Result<AllReceivedJournalistToUserMessages> {
        let mut connection = self.pool.acquire().await?;

        let mut j2u_messages = AllReceivedJournalistToUserMessages::default();

        sqlx::query!(
            r#"
                SELECT
                    user_id AS "user_id: i32",
                    ARRAY_AGG(message) AS "messages: Vec<String>"
                FROM (
                    SELECT
                        user_id,
                        message,
                        RANK() OVER (PARTITION BY user_id ORDER BY received_at DESC)
                    FROM users u
                    LEFT JOIN journalist_to_user_messages j
                        ON u.id = j.user_id
                    WHERE received_at IS NOT NULL
                ) AS x
                WHERE rank <= $1
                GROUP BY 1
            "#,
            MAX_MAILBOX_MESSAGES as i64
        )
        .fetch_all(&mut *connection)
        .await?
        .into_iter()
        .for_each(|row| {
            j2u_messages.insert_user_messages(row.user_id, row.messages.unwrap_or_default());
        });

        Ok(j2u_messages)
    }

    /// Updates the `received_at` timestamp of a user-to-journalist message
    /// that has just been received by the canary. If exactly one row is updated,
    /// the function returns the delivery duration. If no rows are updated,
    /// it means that a duplicate u2j message has been received and the function returns None.
    pub async fn update_u2j_message_setting_received_at(
        &self,
        journalist_id: &JournalistIdentity,
        message: &str,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Option<chrono::Duration>> {
        let mut connection = self.pool.acquire().await?;

        let delivery_duration = sqlx::query_scalar!(
            r#"
                UPDATE user_to_journalist_messages
                SET received_at = $1
                WHERE
                    journalist_id = $2
                    AND message = $3
                    AND received_at IS NULL
                RETURNING
                    sent_at AS "sent_at: DateTime<Utc>"
            "#,
            now,
            journalist_id,
            message,
        )
        .fetch_optional(&mut *connection)
        .await?
        .map(|sent_at| now - sent_at);

        Ok(delivery_duration)
    }

    /// Updates the `received_at` timestamp of a journalist-to-user message
    /// that has just been received by the canary. If exactly one row is updated,
    /// the function returns the delivery duration. If no rows are updated,
    /// it means that a duplicate j2u message has been received and the function returns None.
    pub async fn update_j2u_message_setting_received_at(
        &self,
        user_id: i32,
        journalist_id: &JournalistIdentity,
        message: &str,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Option<chrono::Duration>> {
        let mut connection = self.pool.acquire().await?;

        let delivery_duration = sqlx::query_scalar!(
            r#"
                UPDATE journalist_to_user_messages
                SET received_at = $1
                WHERE
                    journalist_id = $2
                    AND user_id = $3
                    AND message = $4
                    AND received_at IS NULL
                RETURNING sent_at AS "sent_at: DateTime<Utc>"
            "#,
            now,
            journalist_id,
            user_id,
            message
        )
        .fetch_optional(&mut *connection)
        .await?
        .map(|sent_at| now - sent_at);

        Ok(delivery_duration)
    }

    pub async fn get_undelivered_messages(
        &self,
        now: DateTime<Utc>,
        max_delivery_time_hours: u64,
    ) -> anyhow::Result<(i64, i64)> {
        let mut connection = self.pool.acquire().await?;

        // messages older than the validity of a journalist message key are
        // considered undelivered and should be ignored
        let message_key_validity_cutoff = now - JOURNALIST_MSG_KEY_VALID_DURATION;

        let row = sqlx::query!(
            r#"
                WITH u2j AS (
                    SELECT COUNT(*) AS "undelivered_u2j_messages: i64"
                    FROM user_to_journalist_messages
                    WHERE received_at IS NULL
                    AND sent_at < $1
                    AND sent_at > $2
                ), j2u AS (
                    SELECT COUNT(*) AS "undelivered_j2u_messages: i64"
                    FROM journalist_to_user_messages
                    WHERE received_at IS NULL
                    AND sent_at < $1
                    AND sent_at > $2
                )
                SELECT *
                FROM u2j
                CROSS JOIN j2u
            "#,
            now - chrono::Duration::hours(max_delivery_time_hours as i64),
            message_key_validity_cutoff,
        )
        .fetch_one(&mut *connection)
        .await?;

        match (row.undelivered_u2j_messages, row.undelivered_j2u_messages) {
            (Some(undelivered_u2j_messages), Some(undelivered_j2u_messages)) => {
                Ok((undelivered_u2j_messages, undelivered_j2u_messages))
            }
            _ => anyhow::bail!("One or more undelivered message counts are None"),
        }
    }

    //
    // Dead drops
    //

    pub async fn insert_u2j_processed_dead_drop(
        &self,
        dead_drop_id: &i32,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let mut connection = self.pool.acquire().await?;

        sqlx::query!(
            r#"
                INSERT INTO u2j_processed_dead_drops (
                    dead_drop_id,
                    processed_at
                ) VALUES
                ($1, $2)
            "#,
            dead_drop_id,
            now,
        )
        .execute(&mut *connection)
        .await?;

        Ok(())
    }

    pub async fn get_max_u2j_dead_drop_id(&self) -> anyhow::Result<i32> {
        let mut connection = self.pool.acquire().await?;
        let row = sqlx::query!(
            r#"
                SELECT MAX(dead_drop_id) AS "max_dead_drop_id: i32"
                FROM u2j_processed_dead_drops;
            "#
        )
        .fetch_one(&mut *connection)
        .await?;

        Ok(row.max_dead_drop_id.unwrap_or(0))
    }

    pub async fn insert_j2u_processed_dead_drop(
        &self,
        dead_drop_id: &i32,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let mut connection = self.pool.acquire().await?;

        sqlx::query!(
            r#"
                INSERT INTO j2u_processed_dead_drops (
                    dead_drop_id,
                    processed_at
                ) VALUES
                ($1, $2)
            "#,
            dead_drop_id,
            now,
        )
        .execute(&mut *connection)
        .await?;

        Ok(())
    }

    pub async fn get_max_j2u_dead_drop_id(&self) -> anyhow::Result<i32> {
        let mut connection = self.pool.acquire().await?;
        let row = sqlx::query!(
            r#"
                SELECT MAX(dead_drop_id) AS "max_dead_drop_id: i32"
                FROM j2u_processed_dead_drops;
            "#
        )
        .fetch_one(&mut *connection)
        .await?;

        Ok(row.max_dead_drop_id.unwrap_or(0))
    }
}
