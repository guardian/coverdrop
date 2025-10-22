use chrono::{DateTime, Utc};
use std::{
    env, fs,
    path::Path,
    time::{Duration, SystemTime},
};

fn read_fake_time(path: impl AsRef<Path>) -> anyhow::Result<DateTime<Utc>> {
    // Get the time that has elapsed since the fake time was set
    let file_modified = fs::metadata(&path)?.modified()?;
    let system_now = SystemTime::now();

    let duration_since_modified = system_now
        .duration_since(file_modified)
        .unwrap_or(Duration::from_secs(0));

    let duration = chrono::Duration::from_std(duration_since_modified)?;

    // Parse the fake time string
    let time = fs::read_to_string(&path)?;
    let time = DateTime::parse_from_rfc3339(time.trim())?;

    let time_with_elapsed = time + duration;

    Ok(time_with_elapsed.with_timezone(&Utc))
}

/// In order to test many of our expiry or other time-based functions we need to be able to control the time
/// In unit tests this can be done simply by making `now` a parameter to the function, but for our integration
/// tests we need a way of overriding the top-level calls to the `now()` function to move time around.
///
/// Our integration tests run inside Docker containers, which makes direct control of the clocks difficult.
/// To get around this we allow the presence of a environment variable `FAKETIME_TIMESTAMP_FILE` to indicate
/// to our systems that they should read the time out of a file instead of using the normal system clock.
///
/// To use this feature:
/// - Create a file containing an RFC3339 timestamp at a fixed $TIME_PATH
/// - Launch the service with `FAKETIME_TIMESTAMP_FILE=$TIME_PATH`
/// - All calls to `time::now()` will work from that time, plus elapsed time since the file was created (it's important that you don't have a long-lived static time file for this reason)
/// - If you want to time travel, simply write a new ISO time to that file. Subsequent calls to `time::now()` will be based off that time.
///
/// For a practical example of this, see `docker_utils.rs` in the `integration-tests` project.
#[cfg(debug_assertions)]
pub fn now() -> DateTime<Utc> {
    // TODO can we do a custom feature rather than relying on debug_assertions?
    if let Ok(path) = env::var("FAKETIME_TIMESTAMP_FILE") {
        match read_fake_time(path) {
            Ok(time) => {
                return time;
            }
            Err(err) => panic!("Failed to read fake time from file: {err}"),
        }
    }

    Utc::now()
}

#[cfg(not(debug_assertions))]
pub fn now() -> DateTime<Utc> {
    Utc::now()
}

pub fn pretty_print_duration(duration: chrono::Duration) -> String {
    let days = duration.num_days();
    let hours = duration.num_hours() % 24;
    let mins = duration.num_minutes() % 60;

    format!("{days} days {hours} hours {mins} mins")
}

/// Returns a timestamp formatted for use in filenames, e.g. "20251007T133742"
pub fn format_timestamp_for_filename(time: DateTime<Utc>) -> String {
    time.format("%Y%m%d-%H%M%S").to_string()
}
