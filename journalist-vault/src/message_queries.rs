use crate::{
    vault_message::{J2UMessage, U2JMessage, VaultMessage},
    EncryptedJournalistToCoverNodeMessageWithId,
};
use chrono::{DateTime, Duration, Utc};
use common::{
    api::models::{
        dead_drops::DeadDropId, journalist_id::JournalistIdentity,
        messages::journalist_to_covernode_message::EncryptedJournalistToCoverNodeMessage,
    },
    client::mailbox::mailbox_message::UserStatus,
    crypto::keys::encryption::traits::PublicEncryptionKey,
    protocol::keys::UserPublicKey,
    FixedSizeMessageText,
};
use sqlx::SqliteConnection;

pub(crate) async fn add_u2j_message(
    conn: &mut SqliteConnection,
    user_pk: &UserPublicKey,
    message: &FixedSizeMessageText,
    received_at: DateTime<Utc>,
    dead_drop_id: DeadDropId,
) -> anyhow::Result<()> {
    let user_pk_bytes = &user_pk.as_bytes()[..];

    let message_bytes = message.as_bytes();

    sqlx::query!(
        r#"
        INSERT INTO u2j_messages
            (user_pk, message, received_at, dead_drop_id)
        VALUES (?1, ?2, ?3, ?4)
        "#,
        user_pk_bytes,
        message_bytes,
        received_at,
        dead_drop_id
    )
    .execute(conn)
    .await?;

    Ok(())
}

pub(crate) async fn add_j2u_message(
    conn: &mut SqliteConnection,
    user_pk: &UserPublicKey,
    message: &FixedSizeMessageText,
    sent_at: DateTime<Utc>,
    outbound_queue_id: Option<i64>,
) -> anyhow::Result<()> {
    let user_pk_bytes = &user_pk.as_bytes()[..];

    let message_bytes = message.as_bytes();

    sqlx::query!(
        r#"
        INSERT INTO j2u_messages
            (user_pk, message, sent_at, outbound_queue_id)
        VALUES (?1, ?2, ?3, ?4)
        "#,
        user_pk_bytes,
        message_bytes,
        sent_at,
        outbound_queue_id
    )
    .execute(conn)
    .await?;

    Ok(())
}

pub(crate) async fn messages(conn: &mut SqliteConnection) -> anyhow::Result<Vec<VaultMessage>> {
    let messages = sqlx::query!(
        r#"
            WITH messages AS (
                SELECT
                    id,
                    user_pk,
                    message,
                    received_at AS timestamp,
                    read,
                    TRUE AS is_from_user,
                    NULL AS outbound_queue_id
                FROM u2j_messages
                UNION ALL
                SELECT
                    id,
                    user_pk,
                    message,
                    sent_at AS timestamp,
                    NULL AS read,
                    FALSE AS is_from_user,
                    outbound_queue_id
                FROM j2u_messages
            )
            SELECT
                m.id                                   AS "id: i64",
                m.user_pk                              AS "user_pk: Vec<u8>",
                u.alias                                AS "user_alias: String",
                u.description                          AS "user_description: String",
                u.status                               AS "user_status: UserStatus",
                m.is_from_user                         AS "is_from_user: bool",
                m.message                              AS "message: Vec<u8>",
                m.timestamp                            AS "timestamp: DateTime<Utc>",
                m.read                                 AS "read: bool",
                oq.message IS NULL                     AS "is_sent: bool",
                vi.journalist_id                       AS "journalist_id: JournalistIdentity"
            FROM messages m
            CROSS JOIN vault_info vi
            LEFT JOIN outbound_queue oq
                ON oq.id = m.outbound_queue_id
            JOIN users u
                ON u.user_pk = m.user_pk
            ORDER by m.timestamp ASC
        "#
    )
    .try_map(|row| {
        let user_pk = UserPublicKey::from_bytes(&row.user_pk)
            .expect("Parse user_pk into byte array in journalist vault");

        let message = FixedSizeMessageText::from_vec_unchecked(row.message);

        if row.is_from_user {
            Ok(VaultMessage::U2J(U2JMessage::new(
                row.id,
                user_pk,
                row.user_status,
                &message,
                row.timestamp,
                row.read.unwrap_or(false),
                row.user_alias,
                row.user_description,
            )))
        } else {
            Ok(VaultMessage::J2U(J2UMessage::new(
                row.id,
                user_pk,
                row.user_status,
                &message,
                row.is_sent,
                row.timestamp,
                row.user_alias,
                row.user_description,
            )))
        }
    })
    .fetch_all(conn)
    .await?;

    Ok(messages)
}

pub(crate) async fn mark_as_read(
    conn: &mut SqliteConnection,
    message_id: i64,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"UPDATE u2j_messages SET read = true WHERE id = ?1"#,
        message_id
    )
    .execute(conn)
    .await?;

    Ok(())
}

pub(crate) async fn mark_as_unread(
    conn: &mut SqliteConnection,
    message_id: i64,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"UPDATE u2j_messages SET read = false WHERE id = ?1"#,
        message_id
    )
    .execute(conn)
    .await?;

    Ok(())
}

/// Adds a new messages to the FIFO outbound queue.
pub(crate) async fn enqueue_message(
    conn: &mut SqliteConnection,
    message: EncryptedJournalistToCoverNodeMessage,
) -> anyhow::Result<i64> {
    let message = message.as_bytes();

    let queue_id = sqlx::query_scalar!(
        r#"
        INSERT INTO outbound_queue
            (message)
        VALUES (?1)
        RETURNING id"#,
        message
    )
    .fetch_one(conn)
    .await?;

    Ok(queue_id)
}

/// Returns, but does not remove, the front-most (i.e. oldest message) from the FIFO outbound queue.
pub(crate) async fn peek_head_queue_message(
    conn: &mut SqliteConnection,
) -> anyhow::Result<Option<EncryptedJournalistToCoverNodeMessageWithId>> {
    let maybe_message = sqlx::query!(
        r#"
        SELECT
            id       AS "id: i64",
            message  AS "bytes: Vec<u8>"
        FROM outbound_queue
        ORDER BY id ASC
        LIMIT 1"#
    )
    .fetch_optional(conn)
    .await?
    .map(|row| EncryptedJournalistToCoverNodeMessageWithId {
        id: row.id,
        message: EncryptedJournalistToCoverNodeMessage::from_vec_unchecked(row.bytes),
    });

    Ok(maybe_message)
}

/// Deletes the message with the given [id] (see [EncryptedJournalistToCoverNodeMessageWithId]) from the FIFO outbound
/// queue.
pub(crate) async fn delete_queue_message(
    conn: &mut SqliteConnection,
    id: i64,
) -> anyhow::Result<()> {
    sqlx::query!(r#"DELETE FROM outbound_queue WHERE id = $1"#, id)
        .execute(conn)
        .await?;

    Ok(())
}

pub(crate) async fn delete_messages_before(
    conn: &mut SqliteConnection,
    now: DateTime<Utc>,
    message_deletion_duration: Duration,
) -> anyhow::Result<()> {
    let deletion_cutoff = now - message_deletion_duration;
    sqlx::query!(
        r#"
        DELETE FROM u2j_messages
        WHERE received_at < ?1;"#,
        deletion_cutoff,
    )
    .execute(&mut *conn)
    .await?;

    sqlx::query!(
        r#"
        DELETE FROM j2u_messages
        WHERE sent_at < ?1;"#,
        deletion_cutoff,
    )
    .execute(conn)
    .await?;

    Ok(())
}

#[cfg(test)]
mod test {

    use sqlx::pool::PoolConnection;
    use sqlx::Sqlite;

    use common::api::models::messages::journalist_to_covernode_message::EncryptedJournalistToCoverNodeMessage;
    use common::protocol::constants::JOURNALIST_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN;

    use crate::message_queries::{delete_queue_message, enqueue_message, peek_head_queue_message};

    #[sqlx::test]
    async fn test_message_queue_order(mut conn: PoolConnection<Sqlite>) -> sqlx::Result<()> {
        let message_1 = EncryptedJournalistToCoverNodeMessage::from_vec_unchecked(
            [1; JOURNALIST_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN].to_vec(),
        );
        enqueue_message(&mut conn, message_1.clone())
            .await
            .expect("Add first message");

        let message_2 = EncryptedJournalistToCoverNodeMessage::from_vec_unchecked(
            [2; JOURNALIST_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN].to_vec(),
        );
        enqueue_message(&mut conn, message_2.clone())
            .await
            .expect("Add second message");

        let oldest_message = peek_head_queue_message(&mut conn)
            .await
            .expect("Get message")
            .expect("A message to be returned");
        assert_eq!(message_1, oldest_message.message);
        delete_queue_message(&mut conn, oldest_message.id)
            .await
            .expect("Delete message");

        let oldest_message = peek_head_queue_message(&mut conn)
            .await
            .expect("Get message")
            .expect("A message to be returned");
        assert_eq!(message_2, oldest_message.message);
        delete_queue_message(&mut conn, oldest_message.id)
            .await
            .expect("Delete message");

        let oldest_message = peek_head_queue_message(&mut conn)
            .await
            .expect("Get message");
        assert!(oldest_message.is_none());

        Ok(())
    }
}
