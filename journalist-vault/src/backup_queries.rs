use chrono::{DateTime, Utc};
use common::api::models::journalist_id::JournalistIdentity;
use serde::Serialize;
use sqlx::{Decode, SqliteConnection};
use ts_rs::TS;

#[derive(TS, Debug, Serialize, Clone, Decode)]
#[ts(export, rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BackupType {
    Manual,
    Automated,
}

#[derive(Clone, Debug, TS, Serialize)]
#[ts(export, rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct BackupHistoryEntry {
    pub timestamp: DateTime<Utc>,
    pub backup_type: BackupType,
    pub recovery_contacts: Option<Vec<JournalistIdentity>>,
}

pub(crate) async fn record_manual_backup(
    conn: &mut SqliteConnection,
    timestamp: DateTime<Utc>,
    path: &str,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
            INSERT INTO backup_history
                (backup_type, timestamp, path)
            VALUES ('MANUAL', ?1, ?2)
        "#,
        timestamp,
        path
    )
    .execute(conn)
    .await?;

    Ok(())
}

pub(crate) async fn record_automated_backup(
    conn: &mut SqliteConnection,
    timestamp: DateTime<Utc>,
    recovery_contacts: Vec<JournalistIdentity>,
) -> anyhow::Result<()> {
    let recovery_contacts_json = serde_json::to_string(&recovery_contacts)?;

    sqlx::query!(
        r#"
            INSERT INTO backup_history
                (backup_type, timestamp, recovery_contacts)
            VALUES ('AUTOMATED', ?1, ?2)
        "#,
        timestamp,
        recovery_contacts_json
    )
    .execute(conn)
    .await?;

    Ok(())
}

pub(crate) async fn get_backup_history(
    conn: &mut SqliteConnection,
) -> anyhow::Result<Vec<BackupHistoryEntry>> {
    let rows = sqlx::query!(
        r#"
            SELECT
                timestamp AS "timestamp: DateTime<Utc>",
                backup_type AS "backup_type: BackupType",
                recovery_contacts AS "recovery_contacts: String"
            FROM backup_history
            ORDER BY timestamp DESC
        "#
    )
    .fetch_all(conn)
    .await?;

    let backup_history = rows
        .into_iter()
        .map(|r| {
            let recovery_contacts = r
                .recovery_contacts
                .map(|rc| serde_json::from_str::<Vec<JournalistIdentity>>(&rc).unwrap_or_default());
            BackupHistoryEntry {
                timestamp: r.timestamp,
                backup_type: r.backup_type,
                recovery_contacts,
            }
        })
        .collect();

    Ok(backup_history)
}

/// Returns the number of messaging and ID keys created since the last successful backup. For the ID
/// keys it considers both published and candidate keys based on the creation timestamp of the
/// original candidate key pair, i.e. a promoted key is not considered new if its candidate version
/// was created before the last backup.
pub(crate) async fn get_count_of_keys_created_since_last_backup(
    conn: &mut SqliteConnection,
) -> anyhow::Result<i64> {
    let row = sqlx::query!(
        r#" SELECT COUNT(*) AS "count: i64"
            FROM (
                SELECT added_at FROM journalist_msg_key_pairs
                UNION ALL
                SELECT created_at as added_at FROM journalist_id_key_pairs
                UNION ALL
                SELECT added_at FROM candidate_journalist_id_key_pair
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
    let mut query_builder: sqlx::QueryBuilder<'_, sqlx::Sqlite> =
        sqlx::QueryBuilder::new("DELETE FROM backup_contacts;");

    if !contacts.is_empty() {
        query_builder.push(" INSERT INTO backup_contacts (journalist_id) ");
        query_builder.push_values(contacts.iter(), |mut b, contact| {
            b.push_bind(contact);
        });
    }

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
