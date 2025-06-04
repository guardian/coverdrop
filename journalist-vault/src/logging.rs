use std::str::FromStr;

use chrono::{DateTime, Utc};
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
