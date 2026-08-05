use common::api::models::{
    dead_drops::DeadDropId, journalist_id::JournalistIdentity, sentinel_id::SentinelIdentity,
};
use common::clap::Stage;
use sqlx::SqliteConnection;

pub(crate) async fn create_initial_info(
    conn: &mut SqliteConnection,
    journalist_id: &JournalistIdentity,
    sentinel_id: &SentinelIdentity,
    stage: Option<Stage>,
) -> anyhow::Result<()> {
    let stage_str = stage.map(|s| s.as_guardian_str().to_owned());
    sqlx::query!(
        r#"
            INSERT INTO vault_info
                (journalist_id, sentinel_id, max_dead_drop_id, stage, max_delivery_service_message_id)
            VALUES (?1, ?2, 0, ?3, 0)
        "#,
        journalist_id,
        sentinel_id,
        stage_str,
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

/// Returns the stage stored in the vault, if set.
/// Returns None for vaults created before the stage column was added (migration backfill case).
pub(crate) async fn stage(conn: &mut SqliteConnection) -> anyhow::Result<Option<Stage>> {
    let row = sqlx::query!(
        r#"
            SELECT
                stage
            FROM vault_info
        "#
    )
    .fetch_one(conn)
    .await?;

    match row.stage.as_deref() {
        Some(stage_str) => Ok(Some(Stage::from_guardian_str(stage_str)?)),
        None => Ok(None),
    }
}

/// Sets the stage in the vault. This is used to backfill the stage for existing vaults
/// during migration (when the vault is first opened after the migration).
pub(crate) async fn set_stage(conn: &mut SqliteConnection, stage: Stage) -> anyhow::Result<()> {
    let stage_str = stage.as_guardian_str();
    sqlx::query!("UPDATE vault_info SET stage = ?1", stage_str)
        .execute(conn)
        .await?;

    Ok(())
}

pub(crate) async fn max_delivery_service_message_id(
    conn: &mut SqliteConnection,
) -> anyhow::Result<u32> {
    let row = sqlx::query!(
        r#"
            SELECT
                max_delivery_service_message_id
            FROM vault_info
        "#
    )
    .fetch_one(conn)
    .await?;

    Ok(row.max_delivery_service_message_id as u32)
}

pub(crate) async fn sentinel_id(
    conn: &mut SqliteConnection,
) -> anyhow::Result<Option<SentinelIdentity>> {
    let row = sqlx::query!(
        r#"
            SELECT
                sentinel_id AS "sentinel_id: SentinelIdentity"
            FROM vault_info
        "#
    )
    .fetch_one(conn)
    .await?;

    Ok(row.sentinel_id)
}

pub(crate) async fn set_max_delivery_service_message_id(
    conn: &mut SqliteConnection,
    message_id: u32,
) -> anyhow::Result<()> {
    let message_id = message_id as i32;
    sqlx::query!(
        "UPDATE vault_info SET max_delivery_service_message_id = ?1",
        message_id
    )
    .execute(conn)
    .await?;

    Ok(())
}

pub(crate) async fn set_sentinel_id(
    conn: &mut SqliteConnection,
    sentinel_id: &SentinelIdentity,
) -> anyhow::Result<()> {
    sqlx::query!("UPDATE vault_info SET sentinel_id = ?1", sentinel_id,)
        .execute(conn)
        .await?;

    Ok(())
}
