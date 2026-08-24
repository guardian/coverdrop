use chrono::{DateTime, Utc};
use common::crypto::keys::encryption::traits::PublicEncryptionKey;
use common::crypto::keys::encryption::UnsignedEncryptionKeyPair;
use common::protocol::keys::UserPublicKey;
use common::protocol::roles::User;
use common::FixedSizeMessageText;
use journalist_vault::{U2JMessage, VaultMessage};
use sqlx::pool::PoolConnection;
use sqlx::{Row, Sqlite};

const ONE_HOUR: chrono::Duration = chrono::Duration::hours(1);

async fn add_user(
    conn: &mut PoolConnection<Sqlite>,
    user_pk: &UserPublicKey,
    now: DateTime<Utc>,
) -> anyhow::Result<()> {
    let user_pk_bytes = &user_pk.as_bytes()[..];

    sqlx::query(
        r#"
        INSERT INTO users
            (user_pk, status_updated_at)
        VALUES (?1, ?2)
        ON CONFLICT(user_pk) DO NOTHING"#,
    )
    .bind(user_pk_bytes)
    .bind(now)
    .execute(&mut **conn)
    .await
    .expect("insert user before migration should succeed");

    Ok(())
}

async fn add_u2j_message(
    conn: &mut PoolConnection<Sqlite>,
    user_pk: &UserPublicKey,
    message: &FixedSizeMessageText,
    received_at: DateTime<Utc>,
    dead_drop_id: i64,
) -> anyhow::Result<()> {
    let user_pk_bytes = &user_pk.as_bytes()[..];

    let message_bytes = message.as_bytes();

    sqlx::query(
        r#"
        INSERT INTO u2j_messages
            (user_pk, message, received_at, dead_drop_id)
        VALUES (?1, ?2, ?3, ?4)"#,
    )
    .bind(user_pk_bytes)
    .bind(message_bytes)
    .bind(received_at)
    .bind(dead_drop_id)
    .execute(&mut **conn)
    .await
    .expect("insert before migration should succeed");

    Ok(())
}

async fn get_u2j_messages(conn: &mut PoolConnection<Sqlite>) -> anyhow::Result<Vec<VaultMessage>> {
    let messages = sqlx::query(
        r#"
            SELECT
                id,
                user_pk,
                message,
                received_at AS timestamp,
                read
            FROM u2j_messages
            ORDER by received_at ASC
        "#,
    )
    .fetch_all(&mut **conn)
    .await?
    .into_iter()
    .map(|row| {
        let user_pk_bytes: Vec<u8> = row.get("user_pk");
        let user_pk = UserPublicKey::from_bytes(&user_pk_bytes)
            .expect("Parse user_pk into byte array in journalist vault");

        let message_bytes: Vec<u8> = row.get("message");
        let message = FixedSizeMessageText::from_vec_unchecked(message_bytes);

        let id: i64 = row.get("id");
        let timestamp: DateTime<Utc> = row.get("timestamp");
        let read: bool = row.get("read");

        Ok(VaultMessage::U2J(
            U2JMessage::new(id, user_pk, message, timestamp, None, read)
                .expect("Initialize u2j message"),
        ))
    })
    .collect::<Result<Vec<_>, anyhow::Error>>()?;

    Ok(messages)
}

async fn count_u2j_messages(conn: &mut PoolConnection<Sqlite>) -> i64 {
    let row = sqlx::query("SELECT count(*) AS cnt FROM u2j_messages")
        .fetch_one(&mut **conn)
        .await
        .expect("count u2j_messages");
    row.get("cnt")
}

#[sqlx::test(migrations = false)]
async fn test_u2j_message_deduplication_migration(
    mut conn: PoolConnection<Sqlite>,
) -> sqlx::Result<()> {
    // Migrate database version to the schema just before the deduplication migration.
    // TODO once we upgrade to sqlx 0.9.0 we can simply use Migrator::run_to() but for now
    // we need to do run migration sql manually.
    let migrator = sqlx::migrate!();

    let deduplication_migration_id = 20260810115135;

    for migration in migrator
        .iter()
        .filter(|m| m.migration_type.is_up_migration())
    {
        if migration.version == deduplication_migration_id {
            break;
        }
        sqlx::raw_sql(migration.sql.as_ref())
            .execute(&mut *conn)
            .await
            .unwrap_or_else(|e| panic!("Failed to run migration {}: {}", migration.version, e));
    }

    let now: DateTime<Utc> = "2025-07-28T10:30:00Z".parse().unwrap();
    let dead_drop_id = 1;
    let message_1_string = "test message 1";
    let message_2_string = "test message 2";

    let message_1 = FixedSizeMessageText::new(message_1_string).unwrap();
    let message_2 = FixedSizeMessageText::new(message_2_string).unwrap();

    let user_key_pair = UnsignedEncryptionKeyPair::<User>::generate();
    let user_pk = user_key_pair.public_key();
    add_user(&mut conn, user_pk, now)
        .await
        .expect("test user added to DB");

    // Insert two messages with the duplicate user, message text, received at timestamp.
    // These will be deduplicated by the migration.
    add_u2j_message(&mut conn, user_pk, &message_1, now, dead_drop_id)
        .await
        .expect("insert before migration should succeed");

    add_u2j_message(&mut conn, user_pk, &message_1, now, dead_drop_id)
        .await
        .expect("insert before migration should succeed");

    // Add two messages with duplicate user and message text but different received at timestamps.
    // These will NOT be deduplicated by the migration.
    add_u2j_message(&mut conn, user_pk, &message_2, now, dead_drop_id)
        .await
        .expect("insert before migration should succeed");

    add_u2j_message(
        &mut conn,
        user_pk,
        &message_2,
        now + ONE_HOUR, // different received_at timestamp
        dead_drop_id,
    )
    .await
    .expect("insert before migration should succeed");

    // Assert there are 4 messages before the deduplication migration is applied.
    let messages_before_migration = get_u2j_messages(&mut conn).await.unwrap();
    assert_eq!(
        messages_before_migration.len(),
        4,
        "There should be 4 messages before the deduplication migration"
    );

    // Now run just the deduplication migration
    let dedup_migration = migrator
        .iter()
        .find(|m| m.version == deduplication_migration_id && m.migration_type.is_up_migration())
        .expect("deduplication migration exists");
    sqlx::raw_sql(dedup_migration.sql.as_ref())
        .execute(&mut *conn)
        .await
        .expect("deduplication migration runs successfully");

    // Assert results: 1 duplicate removed, so 3 messages remain.
    let count_after = count_u2j_messages(&mut conn).await;
    assert_eq!(
        count_after, 3,
        "There should be 3 messages after the deduplication migration"
    );

    // There is a single copy of message_1, and two copies of message_2 (one with the original
    // received_at timestamp, and one with the different received_at timestamp)
    let all_messages = get_u2j_messages(&mut conn).await.unwrap();
    let message_1_count = all_messages
        .iter()
        .filter(|msg| match msg {
            VaultMessage::U2J(msg) => msg.message == message_1_string,
            VaultMessage::J2U(_) => false,
        })
        .count();
    assert_eq!(
        message_1_count, 1,
        "There should be a single copy of message_1 after deduplication migration"
    );
    let message_2_count = all_messages
        .iter()
        .filter(|msg| match msg {
            VaultMessage::U2J(msg) => msg.message == message_2_string,
            VaultMessage::J2U(_) => false,
        })
        .count();
    assert_eq!(
        message_2_count, 2,
        "There should be two copies of message_2 after deduplication migration"
    );

    Ok(())
}
