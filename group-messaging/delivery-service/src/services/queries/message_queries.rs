use chrono::{DateTime, Utc};
use common::api::models::journalist_id::JournalistIdentity;
use openmls::prelude::{MlsMessageIn, ProtocolMessage};
use sqlx::PgPool;

use delivery_service_lib::models::GroupMessage;
use delivery_service_lib::tls_serialized::TlsSerialized;

use crate::error::DeliveryServiceError;

#[derive(Clone)]
pub struct MessageQueries {
    pool: PgPool,
}

impl MessageQueries {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Atomically store both welcome messages for new members and commit message for existing members.
    /// This ensures that a group membership update is delivered consistently to all parties.
    pub async fn store_add_members_messages(
        &self,
        commit_message: MlsMessageIn,
        existing_members: &[JournalistIdentity],
        new_members: &[JournalistIdentity],
        published_at: DateTime<Utc>,
        welcome_raw_content: &TlsSerialized,
        commit_raw_content: &TlsSerialized,
    ) -> Result<(), DeliveryServiceError> {
        let mut tx = self.pool.begin().await?;

        // Extract protocol message from commit to validate epoch for handshake
        let protocol_msg = ProtocolMessage::try_from(commit_message)
            .map_err(|e| DeliveryServiceError::MlsError(e.to_string()))?;

        // The commit is a handshake message, so validate and update epoch
        if protocol_msg.is_handshake_message() {
            self.validate_and_update_epoch(&mut tx, &protocol_msg, published_at)
                .await?;
        } else {
            return Err(DeliveryServiceError::MalformedMlsMessage(
                "Commit message is expected to be a handshake message with epoch".to_string(),
            ));
        }

        // Store welcome messages for all new members in batch
        if !new_members.is_empty() {
            self.store_message_for_multiple_recipients(
                &mut tx,
                new_members,
                &published_at,
                welcome_raw_content,
            )
            .await?;
        }

        // Store commit message for existing members
        if !existing_members.is_empty() {
            self.store_message_for_multiple_recipients(
                &mut tx,
                existing_members,
                &published_at,
                commit_raw_content,
            )
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    /// Store a group message for multiple recipients.
    /// This also handles epoch validation for handshake messages.
    pub async fn store_group_message(
        &self,
        mls_message: MlsMessageIn,
        recipients: &[JournalistIdentity],
        now: DateTime<Utc>,
        raw_content: &TlsSerialized,
    ) -> Result<(), DeliveryServiceError> {
        let mut tx = self.pool.begin().await?;

        // Extract protocol message to check if it's a handshake message
        let protocol_msg = ProtocolMessage::try_from(mls_message)
            .map_err(|e| DeliveryServiceError::MlsError(e.to_string()))?;

        // If this is a handshake message, validate and update epoch
        if protocol_msg.is_handshake_message() {
            self.validate_and_update_epoch(&mut tx, &protocol_msg, now)
                .await?;
        }

        // Store the message for all recipients in a single batch INSERT
        if !recipients.is_empty() {
            self.store_message_for_multiple_recipients(&mut tx, recipients, &now, raw_content)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    /// Validate that the epoch is not older than what we've seen, and update it.
    /// Returns an error if the epoch is stale.
    /// Uses FOR UPDATE to prevent race conditions between concurrent transactions.
    async fn validate_and_update_epoch(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        protocol_msg: &ProtocolMessage,
        now: DateTime<Utc>,
    ) -> Result<(), DeliveryServiceError> {
        let new_epoch = protocol_msg.epoch().as_u64() as i64;
        let group_id = protocol_msg.group_id().as_slice();

        // Lock the row if it exists to prevent concurrent modifications.
        // Any other transaction that tries to execute this query will be blocked until the lock is released.
        let existing = sqlx::query!(
            r#"
                SELECT epoch
                FROM groups
                WHERE group_id = $1
                FOR UPDATE
            "#,
            group_id
        )
        .fetch_optional(&mut **tx)
        .await?;

        if let Some(existing_row) = existing {
            // Group exists and is now locked - check if new epoch is valid
            if existing_row.epoch >= new_epoch {
                return Err(DeliveryServiceError::StaleEpoch {
                    existing: existing_row.epoch,
                    new: new_epoch,
                });
            }

            // Update to new epoch (row is still locked from FOR UPDATE)
            sqlx::query!(
                r#"
                    UPDATE groups
                    SET epoch = $1, updated_at = $2
                    WHERE group_id = $3
                "#,
                new_epoch,
                now,
                group_id
            )
            .execute(&mut **tx)
            .await?;
        } else {
            // New group - insert it. The PRIMARY KEY constraint ensures uniqueness.
            // If another transaction inserts first, we'll get 0 rows affected and return an error.
            let result = sqlx::query!(
                r#"
                    INSERT INTO groups (group_id, epoch, updated_at)
                    VALUES ($1, $2, $3)
                    ON CONFLICT (group_id) DO NOTHING
                "#,
                group_id,
                new_epoch,
                now
            )
            .execute(&mut **tx)
            .await?;

            // If no rows were inserted, another transaction won the race
            if result.rows_affected() == 0 {
                return Err(DeliveryServiceError::EpochRaceCondition);
            }
        }

        Ok(())
    }

    /// Store a message for multiple recipients using a batch INSERT.
    /// This uses UNNEST with a cross join to duplicate the message and timestamp
    /// for each recipient in a single query.
    async fn store_message_for_multiple_recipients(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        recipients: &[JournalistIdentity],
        published_at: &DateTime<Utc>,
        raw_content: &TlsSerialized,
    ) -> Result<(), DeliveryServiceError> {
        // sqlx text[] parameters require &[String], so we extract the inner strings
        let client_ids: Vec<String> = recipients.iter().map(|id| id.as_ref().clone()).collect();

        sqlx::query!(
            r#"
                INSERT INTO messages (to_client_id, published_at, content)
                SELECT recipient, $2, $3
                FROM UNNEST($1::text[]) AS recipient
            "#,
            &client_ids,
            published_at,
            raw_content.as_ref()
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    /// Retrieve all messages for a client since a given time.
    /// Returns messages ordered by their auto-incrementing id.
    pub async fn get_messages_since(
        &self,
        client_id: &JournalistIdentity,
        ids_greater_than: u32,
    ) -> Result<Vec<GroupMessage>, DeliveryServiceError> {
        let mut connection = self.pool.acquire().await?;

        // Fetch all messages
        let message_rows = sqlx::query!(
            r#"
                SELECT
                    message_id AS "message_id: i32",
                    content AS "content: TlsSerialized",
                    published_at AS "published_at: DateTime<Utc>"
                FROM messages
                WHERE to_client_id = $1
                AND message_id > $2
                ORDER BY message_id ASC
            "#,
            client_id.as_ref(),
            ids_greater_than as i64
        )
        .fetch_all(&mut *connection)
        .await?;

        let messages = message_rows
            .into_iter()
            .map(|row| GroupMessage {
                message_id: row.message_id,
                published_at: row.published_at,
                content: row.content,
            })
            .collect();

        Ok(messages)
    }
}
