use common::api::models::{dead_drops::DeadDropId, journalist_id::JournalistIdentity};
use sqlx::SqliteConnection;

pub(crate) async fn create_initial_info(
    conn: &mut SqliteConnection,
    journalist_id: &JournalistIdentity,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
            INSERT INTO vault_info
                (journalist_id, max_dead_drop_id)
            VALUES (?1, 0)
        "#,
        journalist_id,
    )
    .execute(conn)
    .await?;

    Ok(())
}

pub(crate) async fn journalist_id(
    conn: &mut SqliteConnection,
) -> anyhow::Result<JournalistIdentity> {
    let row = sqlx::query!(
        r#"
            SELECT
                journalist_id AS "id: JournalistIdentity"
            FROM vault_info
        "#
    )
    .fetch_one(conn)
    .await?;

    Ok(row.id)
}

pub(crate) async fn max_dead_drop_id(conn: &mut SqliteConnection) -> anyhow::Result<DeadDropId> {
    let row = sqlx::query!(
        r#"
            SELECT
                max_dead_drop_id AS "max_dead_drop_id: DeadDropId"
            FROM vault_info
        "#
    )
    .fetch_one(conn)
    .await?;

    Ok(row.max_dead_drop_id)
}

pub(crate) async fn set_max_dead_drop_id(
    conn: &mut SqliteConnection,
    dead_drop_id: DeadDropId,
) -> anyhow::Result<()> {
    sqlx::query!("UPDATE vault_info SET max_dead_drop_id = ?1", dead_drop_id)
        .execute(conn)
        .await?;

    Ok(())
}
