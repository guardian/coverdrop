use std::collections::HashMap;

use crate::{
    group_messaging::{GroupId, MessageId},
    GroupMessage, GroupMessageContent, GroupRow, GroupWithMessages,
};
use chrono::{DateTime, Utc};
use common::api::models::sentinel_id::SentinelIdentity;
use sqlx::SqliteConnection;

pub(crate) async fn insert_group(
    conn: &mut SqliteConnection,
    id: &GroupId,
    display_name: Option<&str>,
    description: Option<&str>,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT OR IGNORE INTO groups (id, display_name, description)
        VALUES (?1, ?2, ?3)
        "#,
        id,
        display_name,
        description
    )
    .execute(conn)
    .await?;

    Ok(())
}

pub(crate) async fn get_groups(conn: &mut SqliteConnection) -> anyhow::Result<Vec<GroupRow>> {
    let groups = sqlx::query!(
        r#"
        SELECT
            id              AS "id: GroupId",
            display_name    AS "display_name: String",
            description     AS "description: String"
        FROM groups
        "#
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|row| GroupRow {
        id: row.id,
        display_name: row.display_name,
        description: row.description,
    })
    .collect();

    Ok(groups)
}

pub(crate) async fn get_group(
    conn: &mut SqliteConnection,
    id: &GroupId,
) -> anyhow::Result<GroupRow> {
    let row = sqlx::query!(
        r#"
        SELECT
            id              AS "id: GroupId",
            display_name    AS "display_name: String",
            description     AS "description: String"
        FROM groups
        WHERE id = ?1
        "#,
        id
    )
    .fetch_one(conn)
    .await?;

    Ok(GroupRow {
        id: row.id,
        display_name: row.display_name,
        description: row.description,
    })
}

pub(crate) async fn update_group(
    conn: &mut SqliteConnection,
    id: &GroupId,
    display_name: &String,
    description: &String,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        UPDATE groups
        SET display_name = ?1, description = ?2
        WHERE id = ?3
        "#,
        display_name,
        description,
        id
    )
    .execute(conn)
    .await?;

    Ok(())
}

pub(crate) async fn update_group_name(
    conn: &mut SqliteConnection,
    id: &GroupId,
    display_name: &str,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        UPDATE groups
        SET display_name = ?1
        WHERE id = ?2
        "#,
        display_name,
        id
    )
    .execute(conn)
    .await?;

    Ok(())
}

pub(crate) async fn update_group_description(
    conn: &mut SqliteConnection,
    id: &GroupId,
    description: &String,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        UPDATE groups
        SET description = ?1
        WHERE id = ?2
        "#,
        description,
        id
    )
    .execute(conn)
    .await?;

    Ok(())
}

pub(crate) async fn insert_j2j_message(
    conn: &mut SqliteConnection,
    msg: &GroupMessage,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO j2j_messages (id, group_id, sender_id, message, published_at, read)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        "#,
        msg.id,
        msg.group_id,
        msg.sender,
        msg.content,
        msg.published_at,
        msg.read
    )
    .execute(conn)
    .await?;

    Ok(())
}

pub(crate) async fn update_group_messages_read_status(
    conn: &mut SqliteConnection,
    group_id: &GroupId,
    message_ids: &[MessageId],
    read: bool,
) -> anyhow::Result<()> {
    // sqlx encodes Uuid as BLOB for SQLite, so we bind each id as BLOB via Uuid
    // to match the stored type. Dynamic placeholders are needed since SQLite has
    // no array parameter support.
    let placeholders = message_ids
        .iter()
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(", ");
    let query = format!(
        "UPDATE j2j_messages SET read = ? WHERE group_id = ? AND id IN ({})",
        placeholders
    );
    let mut q = sqlx::query(&query).bind(read).bind(group_id.to_string());
    for id in message_ids {
        q = q.bind(id);
    }
    q.execute(conn).await?;

    Ok(())
}

async fn get_all_messages(conn: &mut SqliteConnection) -> anyhow::Result<Vec<GroupMessage>> {
    let messages = sqlx::query!(
        r#"
        SELECT
            id              AS "id: MessageId",
            group_id        AS "group_id: GroupId",
            sender_id       AS "sender_id: SentinelIdentity",
            message         AS "message: GroupMessageContent",
            published_at    AS "published_at: DateTime<Utc>",
            read            AS "read: bool"
        FROM j2j_messages
        ORDER BY published_at ASC
        "#
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|row| GroupMessage {
        id: row.id,
        group_id: row.group_id,
        sender: row.sender_id,
        content: row.message,
        published_at: row.published_at,
        read: row.read,
    })
    .collect();

    Ok(messages)
}

async fn get_messages_for_group(
    conn: &mut SqliteConnection,
    group_id: &GroupId,
) -> anyhow::Result<Vec<GroupMessage>> {
    let messages = sqlx::query!(
        r#"
        SELECT
            id              AS "id: MessageId",
            group_id        AS "group_id: GroupId",
            sender_id       AS "sender_id: SentinelIdentity",
            message         AS "message: GroupMessageContent",
            published_at    AS "published_at: DateTime<Utc>",
            read            AS "read: bool"
        FROM j2j_messages
        WHERE group_id = ?1
        ORDER BY published_at ASC
        "#,
        group_id
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|row| GroupMessage {
        id: row.id,
        group_id: row.group_id,
        sender: row.sender_id,
        content: row.message,
        published_at: row.published_at,
        read: row.read,
    })
    .collect();

    Ok(messages)
}

pub(crate) async fn get_group_with_messages(
    conn: &mut SqliteConnection,
    id: &GroupId,
) -> anyhow::Result<GroupWithMessages> {
    let group = get_group(conn, id).await?;
    let messages = get_messages_for_group(conn, id).await?;

    Ok(GroupWithMessages {
        id: group.id,
        display_name: group.display_name.unwrap_or_default(),
        description: group.description.unwrap_or_default(),
        messages,
    })
}

pub(crate) async fn get_groups_and_messages(
    conn: &mut SqliteConnection,
) -> anyhow::Result<Vec<GroupWithMessages>> {
    let group_rows = get_groups(conn).await?;
    let all_messages = get_all_messages(conn).await?;

    let mut messages_by_group: HashMap<String, Vec<GroupMessage>> = HashMap::new();
    for message in all_messages {
        messages_by_group
            .entry(message.group_id.to_string())
            .or_default()
            .push(message);
    }

    let groups = group_rows
        .into_iter()
        .map(|row| {
            let messages = messages_by_group
                .remove(&row.id.to_string())
                .unwrap_or_default();
            GroupWithMessages {
                id: row.id,
                display_name: row.display_name.unwrap_or_default(),
                description: row.description.unwrap_or_default(),
                messages,
            }
        })
        .collect();

    Ok(groups)
}
