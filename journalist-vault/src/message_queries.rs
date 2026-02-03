use crate::{
    vault_message::{J2UMessage, U2JMessage, VaultMessage},
    EncryptedJournalistToCoverNodeMessageWithId,
};
use chrono::{DateTime, Duration, Utc};
use common::{
    api::models::{
        dead_drops::DeadDropId,
        messages::journalist_to_covernode_message::EncryptedJournalistToCoverNodeMessage,
    },
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
) -> anyhow::Result<VaultMessage> {
    let user_pk_bytes = &user_pk.as_bytes()[..];

    let message_bytes = message.as_bytes();

    let message_id = sqlx::query_scalar!(
        r#"
        INSERT INTO u2j_messages
            (user_pk, message, received_at, dead_drop_id)
        VALUES (?1, ?2, ?3, ?4)
        RETURNING id"#,
        user_pk_bytes,
        message_bytes,
        received_at,
        dead_drop_id
    )
    .fetch_one(conn)
    .await?;

    Ok(VaultMessage::U2J(U2JMessage::new(
        message_id,
        user_pk.clone(),
        message.clone(),
        received_at,
        None,
        false,
    )?))
}

pub(crate) async fn add_j2u_message(
    conn: &mut SqliteConnection,
    user_pk: &UserPublicKey,
    message: &FixedSizeMessageText,
    sent_at: DateTime<Utc>,
    outbound_queue_id: Option<i64>,
) -> anyhow::Result<VaultMessage> {
    let user_pk_bytes = &user_pk.as_bytes()[..];

    let message_bytes = message.as_bytes();

    let message_id = sqlx::query_scalar!(
        r#"
        INSERT INTO j2u_messages
            (user_pk, message, sent_at, outbound_queue_id)
        VALUES (?1, ?2, ?3, ?4)
        RETURNING id"#,
        user_pk_bytes,
        message_bytes,
        sent_at,
        outbound_queue_id
    )
    .fetch_one(conn)
    .await?;

    Ok(VaultMessage::J2U(J2UMessage::new(
        message_id,
        user_pk.clone(),
        message.clone(),
        false,
        sent_at,
        None,
    )?))
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
                    custom_expiry,
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
                    custom_expiry,
                    NULL AS read,
                    FALSE AS is_from_user,
                    outbound_queue_id
                FROM j2u_messages
            )
            SELECT
                m.id                                   AS "id: i64",
                m.user_pk                              AS "user_pk: Vec<u8>",
                m.is_from_user                         AS "is_from_user: bool",
                m.message                              AS "message: Vec<u8>",
                m.timestamp                            AS "timestamp: DateTime<Utc>",
                m.custom_expiry                        AS "custom_expiry: DateTime<Utc>",
                m.read                                 AS "read: bool",
                oq.message IS NULL                     AS "is_sent: bool"
            FROM messages m
            LEFT JOIN outbound_queue oq
                ON oq.id = m.outbound_queue_id
            ORDER by m.timestamp ASC
        "#
    )
    .try_map(|row| {
        let user_pk = UserPublicKey::from_bytes(&row.user_pk)
            .expect("Parse user_pk into byte array in journalist vault");

        let message = FixedSizeMessageText::from_vec_unchecked(row.message);

        if row.is_from_user {
            Ok(VaultMessage::U2J(
                U2JMessage::new(
                    row.id,
                    user_pk,
                    message,
                    row.timestamp,
                    row.custom_expiry,
                    row.read.unwrap_or(false),
                )
                .expect("Initialize u2j message"),
            ))
        } else {
            Ok(VaultMessage::J2U(
                J2UMessage::new(
                    row.id,
                    user_pk,
                    message,
                    row.is_sent,
                    row.timestamp,
                    row.custom_expiry,
                )
                .expect("Initialize j2u message"),
            ))
        }
    })
    .fetch_all(conn)
    .await?;

    Ok(messages)
}

pub(crate) async fn mark_as_read(
    conn: &mut SqliteConnection,
    user_pk: &UserPublicKey,
) -> anyhow::Result<()> {
    let user_pk_bytes = &user_pk.as_bytes()[..];

    sqlx::query!(
        r#"
            UPDATE u2j_messages
            SET read = true
            WHERE read = false AND user_pk = ?1
    "#,
        user_pk_bytes
    )
    .execute(conn)
    .await?;

    // TODO consider returning the number of messages marked as read
    Ok(())
}

pub(crate) async fn set_custom_expiry(
    conn: &mut SqliteConnection,
    message: &VaultMessage,
    custom_expiry: Option<DateTime<Utc>>,
) -> anyhow::Result<()> {
    match message {
        VaultMessage::J2U(message) => {
            sqlx::query!(
                r#"
        UPDATE j2u_messages
        SET custom_expiry = ?1
        WHERE id = ?2
        "#,
                custom_expiry,
                message.id
            )
            .execute(conn)
            .await?;
        }
        VaultMessage::U2J(message) => {
            sqlx::query!(
                r#"
        UPDATE u2j_messages
        SET custom_expiry = ?1
        WHERE id = ?2
        "#,
                custom_expiry,
                message.id
            )
            .execute(conn)
            .await?;
        }
    }

    Ok(())
}

pub(crate) async fn get_queue_length(conn: &mut SqliteConnection) -> anyhow::Result<i64> {
    let queue_length = sqlx::query_scalar!(r#"SELECT COUNT() FROM outbound_queue"#,)
        .fetch_one(conn)
        .await?;

    Ok(queue_length)
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
        WHERE (received_at < ?1 AND custom_expiry IS NULL) OR custom_expiry < ?2;"#,
        deletion_cutoff,
        now
    )
    .execute(&mut *conn)
    .await?;

    sqlx::query!(
        r#"
        DELETE FROM j2u_messages
        WHERE (sent_at < ?1 AND custom_expiry IS NULL) OR custom_expiry < ?2;"#,
        deletion_cutoff,
        now
    )
    .execute(conn)
    .await?;

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::message_queries::{
        add_j2u_message, add_u2j_message, delete_messages_before, delete_queue_message,
        enqueue_message, messages, peek_head_queue_message, set_custom_expiry,
    };
    use crate::user_queries::add_user;
    use crate::VaultMessage;
    use chrono::{DateTime, Utc};
    use common::api::models::messages::journalist_to_covernode_message::EncryptedJournalistToCoverNodeMessage;
    use common::crypto::keys::encryption::UnsignedEncryptionKeyPair;
    use common::protocol::constants::JOURNALIST_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN;
    use common::protocol::roles::User;
    use common::FixedSizeMessageText;
    use itertools::Itertools;
    use sqlx::pool::PoolConnection;
    use sqlx::Sqlite;

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

    const ONE_HOUR: chrono::Duration = chrono::Duration::hours(1);

    #[sqlx::test]
    async fn test_delete_message_before(mut conn: PoolConnection<Sqlite>) -> sqlx::Result<()> {
        let now: DateTime<Utc> = "2025-07-28T10:30:00Z".parse().unwrap();
        let message_deletion_duration = chrono::Duration::days(14);
        let deletion_cutoff = now - message_deletion_duration;
        let before_cutoff: DateTime<Utc> = deletion_cutoff - ONE_HOUR;
        let after_cutoff: DateTime<Utc> = deletion_cutoff + ONE_HOUR;
        let no_custom_expiry: Option<DateTime<Utc>> = None;
        let custom_expiry_after_now = Some(now + ONE_HOUR);
        let custom_expiry_before_now = Some(now - ONE_HOUR);

        let outbound_queue_id = None;
        let dead_drop_id = 1;

        let message = FixedSizeMessageText::new("test message").unwrap();

        let user_key_pair = UnsignedEncryptionKeyPair::<User>::generate();
        let user_pk = user_key_pair.public_key();
        add_user(&mut conn, user_pk, now)
            .await
            .expect("test user added to DB"); // add the test user to satisfy foreign key constraints

        let _u2j_1 = add_u2j_message(&mut conn, user_pk, &message, before_cutoff, dead_drop_id)
            .await
            .expect("u2j message received before cutoff, so we will expect it to be deleted");
        let _u2j_2 = add_u2j_message(&mut conn, user_pk, &message, after_cutoff, dead_drop_id)
            .await
            .expect("u2j message received after cutoff, so we will expect it not to be deleted");
        let u2j_3 = add_u2j_message(&mut conn, user_pk, &message, after_cutoff, dead_drop_id)
            .await
            .expect("u2j message received after cutoff, but will add custom expiry below");
        set_custom_expiry(
            &mut conn,
            &u2j_3,
            custom_expiry_before_now,
        )
        .await
        .expect(
            "custom expiry set on u2j message 3, to BEFORE now, so we will expect it to be deleted",
        );
        let u2j_4 = add_u2j_message(&mut conn, user_pk, &message, before_cutoff, dead_drop_id)
            .await
            .expect("u2j message received before cutoff, but will add custom expiry below"); // id 4
        set_custom_expiry(
            &mut conn,
            &u2j_4,
            custom_expiry_after_now,
        )
        .await
        .expect("custom expiry set on u2j message 4, to AFTER now, so we will expect it not to be deleted");

        let _j2u_1 = add_j2u_message(
            &mut conn,
            user_pk,
            &message,
            before_cutoff,
            outbound_queue_id,
        )
        .await
        .expect("j2u message sent before cutoff, so we will expect it to be deleted");
        let _j2u_2 = add_j2u_message(
            &mut conn,
            user_pk,
            &message,
            after_cutoff,
            outbound_queue_id,
        )
        .await
        .expect("j2u message sent after cutoff, so we will expect it not to be deleted");
        let j2u_3 = add_j2u_message(
            &mut conn,
            user_pk,
            &message,
            after_cutoff,
            outbound_queue_id,
        )
        .await
        .expect("j2u message sent after cutoff, but will add custom expiry below");
        set_custom_expiry(
            &mut conn,
            &j2u_3,
            custom_expiry_before_now,
        )
        .await
        .expect(
            "custom expiry set on j2u message 3, to BEFORE now, so we will expect it to be deleted",
        );
        let j2u_4 = add_j2u_message(
            &mut conn,
            user_pk,
            &message,
            before_cutoff,
            outbound_queue_id,
        )
        .await
        .expect("j2u message sent before cutoff, but will add custom expiry below");
        set_custom_expiry(
            &mut conn,
            &j2u_4,
            custom_expiry_after_now,
        )
        .await
        .expect("custom expiry set on j2u message 4, to AFTER now, so we will expect it not to be deleted");

        let messages_before_any_deletion = messages(&mut conn).await.unwrap();
        assert_eq!(
            messages_before_any_deletion.len(),
            8,
            "There should be 8 messages before any deletion"
        );

        delete_messages_before(&mut conn, now, message_deletion_duration)
            .await
            .unwrap();

        let messages_after_first_deletion = messages(&mut conn).await.unwrap();
        assert_eq!(
            messages_after_first_deletion.len(),
            4,
            "There should be 4 messages after deletion"
        );
        assert_eq!(
            messages_after_first_deletion
                .iter()
                .map(|msg| match msg {
                    VaultMessage::U2J(msg) => msg.id,
                    VaultMessage::J2U(msg) => msg.id,
                })
                .sorted()
                .collect_vec(),
            vec![2, 2, 4, 4],
            "The messages with id 2 and 4 should remain in both tables"
        );

        delete_messages_before(&mut conn, now, message_deletion_duration)
            .await
            .unwrap();

        let messages_after_second_deletion = messages(&mut conn).await.unwrap();
        assert_eq!(
            messages_after_second_deletion.len(),
            4,
            "There should STILL be 4 messages after second deletion (to simulate job running every min)"
        );

        set_custom_expiry(&mut conn, &u2j_4, no_custom_expiry)
            .await
            .unwrap();
        set_custom_expiry(&mut conn, &j2u_4, no_custom_expiry)
            .await
            .unwrap();

        delete_messages_before(&mut conn, now, message_deletion_duration)
            .await
            .unwrap();

        let messages_after_clearing_some_custom_expiry_and_running_final_deletion =
            messages(&mut conn).await.unwrap();
        assert_eq!(
            messages_after_clearing_some_custom_expiry_and_running_final_deletion.len(),
            2,
            "There should be 2 messages after final deletion (given that we cleared custom expiry for id 4 in each table)"
        );
        assert_eq!(
            messages_after_clearing_some_custom_expiry_and_running_final_deletion
                .iter()
                .map(|msg| match msg {
                    VaultMessage::U2J(msg) => msg.id,
                    VaultMessage::J2U(msg) => msg.id,
                })
                .collect_vec(),
            vec![2, 2],
            "Only messages with id 2 should remain in both tables after final deletion"
        );

        Ok(())
    }
}
