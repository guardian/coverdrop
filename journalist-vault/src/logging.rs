use std::str::FromStr;

use chrono::{DateTime, Duration, Utc};
use sqlx::SqliteConnection;
use tracing::Level;

#[derive(Debug, Clone)]
pub struct Session {
    _id: i64,
    _session_started_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: Level,
    pub target: String,
    pub message: String,
}

impl LogEntry {
    pub fn new(
        timestamp: DateTime<Utc>,
        level: Level,
        target: impl Into<String>,
        message: String,
    ) -> Self {
        Self {
            timestamp,
            level,
            target: target.into(),
            message,
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

pub async fn select_sessions(conn: &mut SqliteConnection) -> anyhow::Result<Vec<Session>> {
    let sessions = sqlx::query_as!(
        Session,
        r#"
        SELECT
            id AS "_id: i64",
            session_started_at AS "_session_started_at: DateTime<Utc>"
        FROM sessions
        ORDER BY session_started_at ASC
        "#,
    )
    .fetch_all(conn)
    .await?;

    Ok(sessions)
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

pub async fn select_log_entries_by_session_range(
    conn: &mut SqliteConnection,
    start_session_id: i64,
    end_session_id: i64,
) -> anyhow::Result<Vec<LogEntry>> {
    let rows = sqlx::query!(
        r#"
        SELECT
            timestamp AS "timestamp: DateTime<Utc>",
            level AS "level: String",
            target AS "target: String",
            message AS "message: String"
        FROM log_entries
        WHERE session_id >= ?1 AND session_id <= ?2
        ORDER BY timestamp ASC
        "#,
        start_session_id,
        end_session_id
    )
    .fetch_all(conn)
    .await?;

    let entries = rows
        .into_iter()
        .map(|row| {
            let level = Level::from_str(&row.level).unwrap_or(Level::INFO);
            LogEntry::new(row.timestamp, level, row.target, row.message)
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
