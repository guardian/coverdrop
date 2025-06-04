use std::{
    fs,
    path::{Path, PathBuf},
};

use common::{
    api::{
        forms::PostSystemStatusEventForm,
        models::general::{StatusEvent, SystemStatus},
    },
    system::keys::AdminKeyPair,
    time,
};
use serde::{Deserialize, Serialize};

use crate::ceremony::SET_SYSTEM_STATUS_AVAILABLE_BUNDLE_FILENAME;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SetSystemStatusAvailableBundle {
    pub set_system_status_form: PostSystemStatusEventForm,
}

/// Saves a collection of upload forms which are used to bootstrap the public key infrastructure
/// and services.
pub fn save_set_system_status_available_bundle(
    output_directory: impl AsRef<Path>,
    admin_key_pair: &AdminKeyPair,
) -> anyhow::Result<PathBuf> {
    assert!(output_directory.as_ref().is_dir());

    let now = time::now();

    let set_system_status_form = PostSystemStatusEventForm::new(
        StatusEvent::new(
            SystemStatus::Available,
            "CoverDrop is available".to_owned(),
            now,
        ),
        admin_key_pair,
        now,
    )?;

    let bundle = SetSystemStatusAvailableBundle {
        set_system_status_form,
    };

    let path = output_directory
        .as_ref()
        .join(SET_SYSTEM_STATUS_AVAILABLE_BUNDLE_FILENAME);

    fs::write(&path, serde_json::to_string_pretty(&bundle)?)?;

    Ok(path)
}
