use common::aws::kinesis::{
    client::StreamKind,
    models::checkpoint::{Checkpoints, CheckpointsJson},
};
use covernode_database::Database;
use std::{
    fs::{self, File},
    path::Path,
};

pub mod checkpoint;
mod controllers;
pub mod key_helpers;
pub mod key_state;
pub mod mixing;
pub mod recipient_tag_lookup_table;
pub mod services;

pub const DEFAULT_PORT: u16 = 3030;

// files for storing the Kinesis stream checkpoints (used only for migration)
const USER_TO_JOURNALIST_CHECKPOINT_FILE: &str = "user_to_journalist_checkpoint.json";
const JOURNALIST_TO_USER_CHECKPOINT_FILE: &str = "journalist_to_user_checkpoint.json";

/// Migrates checkpoint files from the filesystem into the database.
/// After successful migration, the files are deleted.
/// If no checkpoint files exist, this is a no-op.
/// TODO Remove this after migration https://github.com/guardian/coverdrop-internal/issues/4177
pub async fn migrate_checkpoint_files_to_db(
    path: impl AsRef<Path>,
    db: &Database,
) -> anyhow::Result<()> {
    let user_to_journalist_path = path.as_ref().join(USER_TO_JOURNALIST_CHECKPOINT_FILE);
    let journalist_to_user_path = path.as_ref().join(JOURNALIST_TO_USER_CHECKPOINT_FILE);

    // Migrate U2J checkpoint file
    if user_to_journalist_path.exists() {
        let reader = File::open(&user_to_journalist_path)?;
        let checkpoints: Checkpoints = serde_json::from_reader(reader)?;
        let checkpoints_json = CheckpointsJson::new(&checkpoints)?;
        db.update_checkpoint(StreamKind::UserToJournalist, checkpoints_json)
            .await?;
        fs::remove_file(&user_to_journalist_path)?;
        tracing::info!("Migrated U2J checkpoint file to database");
    }

    // Migrate J2U checkpoint file
    if journalist_to_user_path.exists() {
        let reader = File::open(&journalist_to_user_path)?;
        let checkpoints: Checkpoints = serde_json::from_reader(reader)?;
        let checkpoints_json = CheckpointsJson::new(&checkpoints)?;
        db.update_checkpoint(StreamKind::JournalistToUser, checkpoints_json)
            .await?;
        fs::remove_file(&journalist_to_user_path)?;
        tracing::info!("Migrated J2U checkpoint file to database");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::aws::kinesis::models::checkpoint::SequenceNumber;

    #[tokio::test]
    async fn migrate_checkpoint_files_to_db_moves_data_and_deletes_files() {
        let dir = tempfile::tempdir().unwrap();

        // Write checkpoint files with some data
        let mut u2j_checkpoints = Checkpoints::new();
        u2j_checkpoints.insert("shard-001".to_string(), SequenceNumber::from("12345"));
        u2j_checkpoints.insert("shard-002".to_string(), SequenceNumber::from("67890"));
        u2j_checkpoints.insert("shard-003".to_string(), SequenceNumber::from("ABCDE"));

        let mut j2u_checkpoints = Checkpoints::new();
        j2u_checkpoints.insert("shard-001".to_string(), SequenceNumber::from("FGHIJ"));

        let u2j_path = dir.path().join(USER_TO_JOURNALIST_CHECKPOINT_FILE);
        let j2u_path = dir.path().join(JOURNALIST_TO_USER_CHECKPOINT_FILE);

        fs::write(&u2j_path, serde_json::to_string(&u2j_checkpoints).unwrap()).unwrap();
        fs::write(&j2u_path, serde_json::to_string(&j2u_checkpoints).unwrap()).unwrap();

        // Open a temp DB (runs migrations, seeds empty checkpoint rows)
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path, "test-password").await.unwrap();

        // Run the migration
        migrate_checkpoint_files_to_db(dir.path(), &db)
            .await
            .unwrap();

        // Files should be deleted
        assert!(!u2j_path.exists());
        assert!(!j2u_path.exists());

        // DB should have the checkpoint data
        let stored = db.select_checkpoints().await.unwrap();
        assert_eq!(stored.user_to_journalist_checkpoints, u2j_checkpoints);
        assert_eq!(stored.journalist_to_user_checkpoints, j2u_checkpoints);
    }

    #[tokio::test]
    async fn migrate_checkpoint_files_is_noop_when_no_files_exist() {
        let dir = tempfile::tempdir().unwrap();

        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path, "test-password").await.unwrap();

        // Should succeed with no files present
        migrate_checkpoint_files_to_db(dir.path(), &db)
            .await
            .unwrap();

        // DB should have empty checkpoints (from migration seed)
        let stored = db.select_checkpoints().await.unwrap();
        assert_eq!(stored.user_to_journalist_checkpoints, Checkpoints::new());
        assert_eq!(stored.journalist_to_user_checkpoints, Checkpoints::new());
    }
}
