use common::aws::kinesis::{
    client::StreamKind,
    models::checkpoint::{Checkpoints, CheckpointsJson, StoredCheckpoints},
};
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

// files for storing the Kinesis stream checkpoints
const USER_TO_JOURNALIST_CHECKPOINT_FILE: &str = "user_to_journalist_checkpoint.json";
const JOURNALIST_TO_USER_CHECKPOINT_FILE: &str = "journalist_to_user_checkpoint.json";

fn write_empty_checkpoint_file(path: impl AsRef<Path>) -> anyhow::Result<()> {
    let checkpoint = serde_json::to_string(&Checkpoints::new())?;

    fs::write(path, checkpoint)?;

    Ok(())
}

pub fn load_checkpoints(path: impl AsRef<Path>) -> anyhow::Result<StoredCheckpoints> {
    let user_to_journalist_path = path.as_ref().join(USER_TO_JOURNALIST_CHECKPOINT_FILE);
    let journalist_to_user_path = path.as_ref().join(JOURNALIST_TO_USER_CHECKPOINT_FILE);

    // ------------------------------------------------------------------------------------------
    // Migration code can be removed once we've migrated all existing production checkpoint files
    let old_user_to_journalist_path = path.as_ref().join("user_checkpoint.json");
    let old_journalist_to_user_path = path.as_ref().join("journalist_checkpoint.json");

    if old_user_to_journalist_path.exists() {
        fs::rename(&old_user_to_journalist_path, &user_to_journalist_path)?;
    }
    if old_journalist_to_user_path.exists() {
        fs::rename(&old_journalist_to_user_path, &journalist_to_user_path)?;
    }
    // END migration code
    // ------------------------------------------------------------------------------------------

    if !user_to_journalist_path.exists() {
        write_empty_checkpoint_file(&user_to_journalist_path)?;
    }

    if !journalist_to_user_path.exists() {
        write_empty_checkpoint_file(&journalist_to_user_path)?;
    }

    let reader = File::open(user_to_journalist_path)?;
    let user_to_journalist_checkpoints = serde_json::from_reader::<_, Checkpoints>(reader)?;

    let reader = File::open(journalist_to_user_path)?;
    let journalist_to_user_checkpoints = serde_json::from_reader::<_, Checkpoints>(reader)?;

    Ok(StoredCheckpoints {
        user_to_journalist_checkpoints,
        journalist_to_user_checkpoints,
    })
}

pub fn update_checkpoint(
    path: impl AsRef<Path>,
    stream_kind: StreamKind,
    checkpoints_json: CheckpointsJson,
) -> anyhow::Result<()> {
    let mut path = path.as_ref().to_owned();
    let json_path = match stream_kind {
        StreamKind::UserToJournalist => USER_TO_JOURNALIST_CHECKPOINT_FILE,
        StreamKind::JournalistToUser => JOURNALIST_TO_USER_CHECKPOINT_FILE,
    };
    path.push(json_path);

    fs::write(path, checkpoints_json)?;

    Ok(())
}
