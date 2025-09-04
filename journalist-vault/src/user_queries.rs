use crate::User;
use chrono::{DateTime, Utc};
use common::{
    client::mailbox::mailbox_message::UserStatus,
    crypto::keys::encryption::traits::PublicEncryptionKey, protocol::keys::UserPublicKey,
};
use sqlx::SqliteConnection;

pub(crate) async fn user_pks(
    conn: &mut SqliteConnection,
) -> anyhow::Result<impl Iterator<Item = UserPublicKey>> {
    let user_pks = sqlx::query!(
        r#"
            SELECT user_pk AS "user_pk: Vec<u8>"
            FROM users
        "#
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .flat_map(|row| UserPublicKey::from_bytes(&row.user_pk));

    Ok(user_pks)
}

pub(crate) async fn users(conn: &mut SqliteConnection) -> anyhow::Result<Vec<User>> {
    let users = sqlx::query!(
        r#"
            SELECT
                user_pk AS "user_pk: Vec<u8>",
                alias AS "alias: String",
                description AS "description: String",
                status AS "status: UserStatus",
                marked_as_unread AS "marked_as_unread: bool"
            FROM users
        "#
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|row| {
        let user_pk = UserPublicKey::from_bytes(&row.user_pk)?;
        Ok(User {
            user_pk,
            status: row.status,
            alias: row.alias,
            description: row.description,
            marked_as_unread: row.marked_as_unread,
        })
    })
    .collect::<anyhow::Result<Vec<_>>>()?;

    Ok(users)
}

pub(crate) async fn add_user(
    conn: &mut SqliteConnection,
    user_pk: &UserPublicKey,
    now: DateTime<Utc>,
) -> anyhow::Result<()> {
    let user_pk_bytes = &user_pk.as_bytes()[..];

    sqlx::query!(
        r#"
            INSERT INTO users
                (user_pk, status_updated_at)
            VALUES (?1, ?2)
            ON CONFLICT(user_pk) DO NOTHING
        "#,
        user_pk_bytes,
        now
    )
    .execute(conn)
    .await?;

    Ok(())
}

pub(crate) async fn update_user_status(
    conn: &mut SqliteConnection,
    user_pk: &UserPublicKey,
    status: UserStatus,
) -> anyhow::Result<()> {
    let user_pk_bytes = &user_pk.as_bytes()[..];
    let status_string = status.to_string();

    sqlx::query!(
        r#"
            UPDATE users
            SET status = ?1
            WHERE user_pk = ?2"#,
        status_string,
        user_pk_bytes
    )
    .execute(conn)
    .await?;

    Ok(())
}

pub(crate) async fn mark_as_read(
    conn: &mut SqliteConnection,
    user_pk: &UserPublicKey,
) -> anyhow::Result<()> {
    let user_pk_bytes = &user_pk.as_bytes()[..];

    sqlx::query!(
        r#"UPDATE users SET marked_as_unread = false WHERE user_pk = ?1"#,
        user_pk_bytes
    )
    .execute(conn)
    .await?;

    Ok(())
}

pub(crate) async fn mark_as_unread(
    conn: &mut SqliteConnection,
    user_pk: &UserPublicKey,
) -> anyhow::Result<()> {
    let user_pk_bytes = &user_pk.as_bytes()[..];

    sqlx::query!(
        r#"UPDATE users SET marked_as_unread = true WHERE user_pk = ?1"#,
        user_pk_bytes
    )
    .execute(conn)
    .await?;

    Ok(())
}

pub(crate) async fn update_user_alias_and_description(
    conn: &mut SqliteConnection,
    user_pk: &UserPublicKey,
    alias: &str,
    description: &str,
) -> anyhow::Result<()> {
    let user_pk_bytes = &user_pk.as_bytes()[..];

    let query = sqlx::query!(
        r#"
            UPDATE users
            SET alias = ?1,
            description = ?2
            WHERE user_pk = ?3"#,
        alias,
        description,
        user_pk_bytes
    )
    .execute(conn)
    .await?;

    if query.rows_affected() == 0 {
        return Err(anyhow::anyhow!(
            "Failed to update user alias and description for user: {}",
            hex::encode(user_pk_bytes)
        ));
    }

    Ok(())
}
