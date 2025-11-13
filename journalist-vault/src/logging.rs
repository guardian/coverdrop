use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqliteConnection;
use ts_rs::TS;

#[derive(Clone, Serialize, Deserialize, TS)]
#[ts(export, rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct LoggingSession {
    pub session_id: i64,
    pub min_timestamp: DateTime<Utc>,
    pub max_timestamp: DateTime<Utc>,
}

impl LoggingSession {
    pub fn new(
        session_id: i64,
        min_timestamp: DateTime<Utc>,
        max_timestamp: DateTime<Utc>,
    ) -> Self {
        Self {
            session_id,
            min_timestamp,
            max_timestamp,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, TS)]
#[ts(export, rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub id: Option<i64>,
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub target: String,
    pub message: String,
    pub session_id: Option<i64>,
}

impl LogEntry {
    pub fn new(
        id: Option<i64>,
        timestamp: DateTime<Utc>,
        level: String,
        target: impl Into<String>,
        message: String,
        session_id: Option<i64>,
    ) -> Self {
        Self {
            id,
            timestamp,
            level,
            target: target.into(),
            message,
            session_id,
        }
    }

    pub fn to_log_string(&self) -> String {
        format!(
            "[{}] {} {}: {}",
            self.timestamp, self.level, self.target, self.message
        )
    }

    /// Get the size of the log entry in bytes
    pub fn size(&self) -> usize {
        std::mem::size_of::<Self>() + self.message.capacity() + self.target.capacity()
    }
}

pub async fn insert_session(
    conn: &mut SqliteConnection,
    session_started_at: DateTime<Utc>,
) -> anyhow::Result<i64> {
    let result = sqlx::query_scalar!(
        r#"
            INSERT INTO sessions (session_started_at)
            VALUES (?1)
            RETURNING id AS "id: i64"
        "#,
        session_started_at
    )
    .fetch_one(conn)
    .await?;

    Ok(result)
}

pub async fn insert_log_entries(
    conn: &mut SqliteConnection,
    session_id: i64,
    log_entries: &[LogEntry],
) -> anyhow::Result<()> {
    for log_entry in log_entries {
        let level = log_entry.level.to_string();

        sqlx::query!(
            r#"
            INSERT INTO log_entries (session_id, timestamp, level, target, message)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
            session_id,
            log_entry.timestamp,
            level,
            log_entry.target,
            log_entry.message,
        )
        .execute(&mut *conn)
        .await?;
    }

    Ok(())
}

pub async fn get_session_timeline(
    conn: &mut SqliteConnection,
) -> anyhow::Result<Vec<LoggingSession>> {
    let rows = sqlx::query!(
        r#"
        SELECT
            session_id AS "session_id: i64",
            MIN(timestamp) AS "min_timestamp: DateTime<Utc>",
            MAX(timestamp) AS "max_timestamp: DateTime<Utc>"
        FROM log_entries
        GROUP BY session_id
        ORDER BY session_id DESC
        "#
    )
    .fetch_all(conn)
    .await?;

    let sessions = rows
        .into_iter()
        .map(|row| {
            LoggingSession::new(
                row.session_id.expect("missing session_id"),
                row.min_timestamp,
                row.max_timestamp,
            )
        })
        .collect();

    Ok(sessions)
}

pub async fn select_log_entries(
    conn: &mut SqliteConnection,
    min_level: String,
    search_term: String,
    before: DateTime<Utc>,
    limit: i64,
    offset: i64,
) -> anyhow::Result<Vec<LogEntry>> {
    let search_term_like_pattern = format!("%{search_term}%");
    let rows = sqlx::query!(
        r#"
        SELECT
            ROWID AS "id: i64",
            timestamp AS "timestamp: DateTime<Utc>",
            level AS "level: String",
            target AS "target: String",
            message AS "message: String",
            session_id AS "session_id: i64"
        FROM log_entries
        WHERE timestamp < $1
            AND (message LIKE $2 OR target LIKE $2)
            AND CASE $3
                WHEN 'TRACE' THEN TRUE
                WHEN 'DEBUG' THEN level IN ('DEBUG', 'INFO', 'WARN', 'ERROR')
                WHEN 'INFO' THEN level IN ('INFO', 'WARN', 'ERROR')
                WHEN 'WARN' THEN level IN ('WARN', 'ERROR')
                WHEN 'ERROR' THEN level = 'ERROR'
                ELSE FALSE
        END
        ORDER BY timestamp DESC
        LIMIT $4 OFFSET $5
        "#,
        before,
        search_term_like_pattern,
        min_level,
        limit,
        offset
    )
    .fetch_all(conn)
    .await?;

    let entries = rows
        .into_iter()
        .map(|row| {
            LogEntry::new(
                row.id,
                row.timestamp,
                row.level,
                row.target,
                row.message,
                Some(row.session_id),
            )
        })
        .collect();

    Ok(entries)
}

/// Delete all logging sessions and their associated log entries that are older than 2 weeks
/// and retain only the most recent 15k DEBUG and the most recent 5k TRACE (i.e. roughly 4MB in total)
pub async fn delete_old_logs(
    conn: &mut SqliteConnection,
    now: DateTime<Utc>,
) -> anyhow::Result<()> {
    // Use checked_sub to safely handle potential underflow
    let two_weeks_ago = now
        .checked_sub_signed(Duration::weeks(2))
        .unwrap_or(DateTime::<Utc>::MIN_UTC);

    // First delete log entries for old sessions (to maintain foreign key constraints)
    sqlx::query!(
        r#"
        DELETE FROM log_entries
        WHERE session_id IN (
            SELECT id FROM sessions WHERE session_started_at < ?1
        )
        "#,
        two_weeks_ago
    )
    .execute(&mut *conn)
    .await?;

    // Then delete the old sessions
    sqlx::query!(
        r#"
        DELETE FROM sessions
        WHERE session_started_at < ?1
        "#,
        two_weeks_ago
    )
    .execute(&mut *conn)
    .await?;

    // Pick up any remaining log entries. Should only do something if the journalist
    // never closes their vault so their session is more than two weeks old.
    // This is unfortunately another query because we don't cascade deletes of
    // sessions to log entries, otherwise we could just do this as our first query.
    sqlx::query!(
        r#"
        DELETE FROM log_entries
        WHERE timestamp < ?1
        "#,
        two_weeks_ago
    )
    .execute(&mut *conn)
    .await?;

    // by analysing some vaults, 5k logs equate to roughly 1MB in vault size
    // TRACE and DEBUG logs are quite noisy and only really useful if very recent (in case of freeze/crash)
    // so delete all but the most recent 15k DEBUG and the most recent 5k TRACE (i.e. roughly 4MB in total)
    sqlx::query!(
        r#"
        DELETE FROM log_entries WHERE ROWID IN (
            SELECT ROWID
            FROM log_entries
            WHERE level = 'DEBUG'
            ORDER BY timestamp DESC
            LIMIT -1 OFFSET 15000
        )
        "#
    )
    .execute(&mut *conn)
    .await?;
    sqlx::query!(
        r#"
        DELETE FROM log_entries WHERE ROWID IN (
            SELECT ROWID
            FROM log_entries
            WHERE level = 'TRACE'
            ORDER BY timestamp DESC
            LIMIT -1 OFFSET 5000
        )
        "#
    )
    .execute(&mut *conn)
    .await?;

    Ok(())
}
