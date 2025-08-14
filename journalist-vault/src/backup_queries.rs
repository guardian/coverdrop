use chrono::{DateTime, Utc};
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
