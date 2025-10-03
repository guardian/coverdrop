use chrono::{DateTime, Utc};
use common::api::models::journalist_id::JournalistIdentity;
use sqlx::SqliteConnection;

pub(crate) async fn record_successful_backup(
    conn: &mut SqliteConnection,
    timestamp: DateTime<Utc>,
    path: &str,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
            INSERT INTO backup_history
                (timestamp, path)
            VALUES (?1, ?2)
        "#,
        timestamp,
        path
    )
    .execute(conn)
    .await?;

    Ok(())
}

pub(crate) async fn get_count_of_keys_created_since_last_backup(
    conn: &mut SqliteConnection,
) -> anyhow::Result<i64> {
    let row = sqlx::query!(
        r#" SELECT COUNT(*) AS "count: i64"
            FROM (
                SELECT added_at FROM journalist_msg_key_pairs
                UNION ALL
                SELECT added_at FROM journalist_id_key_pairs
            )
            WHERE (NOT EXISTS(SELECT 1 FROM backup_history)) OR added_at > (SELECT MAX(timestamp) FROM backup_history)
        "#
    )
    .fetch_one(conn)
    .await?;

    Ok(row.count)
}

pub(crate) async fn get_backup_contacts(
    conn: &mut SqliteConnection,
) -> anyhow::Result<Vec<JournalistIdentity>> {
    let rows = sqlx::query!(
        r#"
            SELECT journalist_id AS "journalist_id: JournalistIdentity"
            FROM backup_contacts
        "#
    )
    .fetch_all(conn)
    .await?;

    Ok(rows.into_iter().map(|r| r.journalist_id).collect())
}

pub(crate) async fn set_backup_contacts(
    conn: &mut SqliteConnection,
    contacts: Vec<JournalistIdentity>,
) -> anyhow::Result<()> {
    let mut query_builder: sqlx::QueryBuilder<'_, sqlx::Sqlite> = sqlx::QueryBuilder::new(
        "DELETE FROM backup_contacts; INSERT INTO backup_contacts (journalist_id) ",
    );

    query_builder.push_values(contacts.iter(), |mut b, contact| {
        b.push_bind(contact);
    });

    query_builder.build().execute(conn).await?;

    Ok(())
}

pub(crate) async fn remove_invalid_backup_contacts(
    conn: &mut SqliteConnection,
    journalist_identities_from_api: Vec<&JournalistIdentity>,
) -> anyhow::Result<u64> {
    let contacts = serde_json::to_string(&journalist_identities_from_api)?;
    let result = sqlx::query!(
        r#"
            DELETE FROM backup_contacts
            WHERE journalist_id NOT IN (
                SELECT value FROM json_each(?1)
            )
        "#,
        contacts
    )
    .execute(conn)
    .await?;

    Ok(result.rows_affected())
}
