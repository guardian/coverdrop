use std::path::Path;

use chrono::{DateTime, Utc};
use testcontainers::{
    core::{ExecCommand, Mount},
    ContainerAsync, Image,
};

/// The `testcontainers` library requires a string tuple of `(bind_mount, volume_path)` to bind a directory
/// to the Docker container. This function converts a well typed `Path` and a path within the volume to that tuple.
pub fn temp_dir_to_mount(dir_path: impl AsRef<Path>, volume_path: &str) -> Mount {
    let bind_mount = dir_path
        .as_ref()
        .as_os_str()
        .to_str()
        .expect("Convert keys_path to String")
        .to_owned();

    Mount::bind_mount(bind_mount, volume_path.to_owned())
}

/// Create a bash command to set the faketime of a container using a provided `DateTime`.
pub fn date_time_to_set_faketime_command_string(time: DateTime<Utc>) -> String {
    format!("/bin/echo '{}' > /faketime", time.to_rfc3339())
}

/// Time travel a container to a specific point in time. Requires the package to be using the `common::time::now()` command to get the current time.
pub async fn time_travel_container<T: Image>(container: &ContainerAsync<T>, to: DateTime<Utc>) {
    let date_time_command = date_time_to_set_faketime_command_string(to);
    // By default docker containers are in the UTC timezone
    // it's important that our real infrastructure is also in UTC.
    let cmd = ExecCommand::new(vec!["bash", "-c", &date_time_command]);

    container.exec(cmd).await.expect("Time travel");
}
