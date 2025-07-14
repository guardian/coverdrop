use std::io::Write;
use std::{fs::File, path::PathBuf};

use testcontainers::ContainerAsync;

//
// NOTE!!!
// =======
//
// These secrets MUST be set to values that are unlikely to appear in the logs
// normally because we're going to scan our logs looking for them. E.g. if
// you were to change one of these values to 'password' then our secret scanner would
// trigger since there are many logging messages talking about passwords.
//
pub const API_AWS_ACCESS_KEY_ID_SECRET: &str = "API_AWS_ACCESS_KEY_ID_SECRET";
pub const API_AWS_SECRET_ACCESS_KEY_SECRET: &str = "API_AWS_SECRET_ACCESS_KEY_SECRET";

// This is consider private but not secret
pub const COVERNODE_AWS_ACCESS_KEY_ID_SECRET: &str = "COVERNODE_AWS_ACCESS_KEY_ID_SECRET";
pub const COVERNODE_AWS_SECRET_ACCESS_KEY_SECRET: &str = "COVERNODE_AWS_SECRET_ACCESS_KEY_SECRET";
// This is consider private but not secret
pub const IDENTITY_API_AWS_ACCESS_KEY_ID_SECRET: &str = "IDENTITY_API_AWS_ACCESS_KEY_ID_SECRET";
pub const IDENTITY_API_AWS_SECRET_ACCESS_KEY_SECRET: &str =
    "IDENTITY_API_AWS_SECRET_ACCESS_KEY_SECRET";
// This is consider private but not secret
pub const COVERDROP_POSTGRES_DB_SECRET: &str = "COVERDROP_POSTGRES_DB_SECRET";
pub const MAILBOX_PASSWORD: &str = "zoom zoom zoom zoom zoom";

pub fn get_all_secrets() -> Vec<String> {
    vec![
        COVERNODE_AWS_SECRET_ACCESS_KEY_SECRET.to_string(),
        COVERDROP_POSTGRES_DB_SECRET.to_string(),
        IDENTITY_API_AWS_SECRET_ACCESS_KEY_SECRET.to_string(),
    ]
}

pub async fn do_secrets_exist_in_container_logs<T: testcontainers::Image>(
    container: &ContainerAsync<T>,
    log_path: PathBuf,
) -> Result<bool, anyhow::Error> {
    let combined_logs = container.stdout_to_vec().await?;
    let mut combined_logs = String::from_utf8(combined_logs).expect("Parse UTF-8");
    let error_logs = container.stderr_to_vec().await?;
    let error_logs = String::from_utf8(error_logs).expect("Parse UTF-8");
    combined_logs.push_str(&error_logs);

    tracing::debug!("checking secrets");

    let secret_count: usize = get_all_secrets()
        .iter()
        .filter(|&secret| {
            let logs_contain_secret = combined_logs.contains(secret);
            if logs_contain_secret {
                tracing::debug!("secret {} found in logs", secret);
            }
            logs_contain_secret
        })
        .count();

    tracing::debug!("got secrets count: {}", secret_count);

    let logs_writer = File::create(log_path).expect("Create log file");
    write!(&logs_writer, "{combined_logs}").unwrap();
    Ok(secret_count > 0)
}
