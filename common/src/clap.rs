use std::{
    fmt::{self, Display},
    fs::read_to_string,
    marker::PhantomData,
    ops::Deref,
    path::PathBuf,
    str::FromStr,
};

use clap::{Args, ValueEnum};
use rpassword::prompt_password;

use crate::generators::PasswordGenerator;

/// Common AWS configuration parameters such as the region.
#[derive(Args, Clone, Debug)]
pub struct AwsConfig {
    /// AWS region
    #[clap(
        name = "aws-region",
        long,
        env = "AWS_REGION",
        global = true,
        default_value = "eu-west-1"
    )]
    pub region: String,
    /// AWS profile
    #[clap(name = "aws-profile", long, env = "AWS_PROFILE", global = true)]
    pub profile: Option<String>,
}

/// Shared Kinesis configuration parameters consisting of the endpoint and the names of the
/// user(-to-journalist) and the journalist(-to-user) streams.
#[derive(Args, Clone, Debug)]
pub struct KinesisConfig {
    /// The address of the Kinesis stream endpoint
    #[clap(name = "kinesis-endpoint", long, env = "KINESIS_ENDPOINT")]
    pub endpoint: String,
    /// The name of the Kinesis stream containing user messages
    #[clap(name = "kinesis-user-stream", long, env = "KINESIS_USER_STREAM")]
    pub user_stream: String,
    /// The name of the Kinesis stream containing journalist messages
    #[clap(
        name = "kinesis-journalist-stream",
        long,
        env = "KINESIS_JOURNALIST_STREAM"
    )]
    pub journalist_stream: String,
}

//
// Secrets redaction
// It is nice to be able to debug print our CLI arguments on startup as this can save a lot of
// time when investigating an issue. Some of our CLI arguments are secrets though, so we want a way
// of hiding those values from `Debug` calls without introducing overheads in the developer experience.
//
// For this we have `CliSecret` and `RedactionFunction` which you can wrap around values which will modify
// debug print output.
//

pub trait RedactionFunction<T> {
    fn redact(s: &T) -> String;
}

const REDACTED_VALUE: &str = "<REDACTED>";

#[derive(Clone)]
pub struct PostgresConnectionStringRedactor {}

impl RedactionFunction<String> for PostgresConnectionStringRedactor {
    fn redact(s: &String) -> String {
        const INVALID_CONNECTION_STRING_REDACTION: &str =
            "<REDACTED IN WHOLE DUE TO UNRECOGNIZED CONNECTION STRING FORMAT>";

        // Verify our string is a postgres connection string that we can handle
        if !s.starts_with("postgresql://") {
            return INVALID_CONNECTION_STRING_REDACTION.to_string();
        }

        const PROTOCOL_STRING_LEN: usize = "postgresql://".len();

        // Find the separator between the username:password pair and the host, an at character.
        let Some(password_end) = s.find('@') else {
            return INVALID_CONNECTION_STRING_REDACTION.to_string();
        };

        // Skip over the first colon (from the postgres protocol string) and find the next colo.
        // That is the username:password separator.
        // Note this will find the offset within the *substring*
        let Some(password_start) = s[PROTOCOL_STRING_LEN..password_end].find(':') else {
            return INVALID_CONNECTION_STRING_REDACTION.to_string();
        };

        let password_start = password_start
            + PROTOCOL_STRING_LEN // Add the protocol string length back
            + 1; // Add one to jump over the colon

        let mut cleaned_db_url = s.clone();
        cleaned_db_url.replace_range(password_start..password_end, REDACTED_VALUE);
        cleaned_db_url
    }
}

#[derive(Clone)]
pub struct PlainRedactor {}

impl<T> RedactionFunction<T> for PlainRedactor {
    fn redact(_: &T) -> String {
        REDACTED_VALUE.to_string()
    }
}

#[derive(Clone)]
pub struct CliSecret<T, R>
where
    R: RedactionFunction<T>,
{
    value: T,
    redaction_function: PhantomData<R>,
}

impl<T, R> CliSecret<T, R>
where
    R: RedactionFunction<T>,
{
    pub fn new(value: T) -> Self {
        Self {
            value,
            redaction_function: PhantomData,
        }
    }
}

impl<T, R> Deref for CliSecret<T, R>
where
    R: RedactionFunction<T>,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T, R> fmt::Debug for CliSecret<T, R>
where
    T: Display,
    R: RedactionFunction<T>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = R::redact(&self.value);
        f.write_str(&text)
    }
}

impl<T, R> FromStr for CliSecret<T, R>
where
    T: FromStr,
    R: RedactionFunction<T>,
{
    type Err = T::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let t = T::from_str(s)?;
        Ok(CliSecret {
            value: t,
            redaction_function: PhantomData,
        })
    }
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum Stage {
    #[clap(aliases = &["prod", "PROD", "PRODUCTION"])]
    Production,
    #[clap(aliases = &["STAGING"])]
    Staging,
    #[clap(aliases = &["CODE"])]
    Code,
    #[clap(aliases = &["dev", "DEV", "DEVELOPMENT"])]
    Development,
}

impl Stage {
    /// Converts the `Stage` to a string slice following the Guardian's
    /// stage naming convention. This is required for external systems
    /// such as our metrics aggregators and AWS tags. In the future we
    /// could come up with another abstraction that allows other conventions
    /// to be followed so that this is less Guardian specific.
    pub fn as_guardian_str(&self) -> &'static str {
        match self {
            Stage::Production => "PROD",
            Stage::Staging => "STAGING",
            Stage::Code => "CODE",
            Stage::Development => "DEV",
        }
    }

    /// Converts the `Stage` into a string slice showing the most convenient name
    /// for typing on the command line. Useful in help messages.
    pub fn as_clap_str(&self) -> &'static str {
        match self {
            Stage::Production => "prod",
            Stage::Staging => "staging",
            Stage::Code => "code",
            Stage::Development => "dev",
        }
    }
}

/// Get a password from the user, either by prompting them, reading it from args, or reading it from a file.
///
/// If both a raw password string and a path to a password file are provided then the raw password string
/// will be used.
pub fn validate_password_from_args(
    password: Option<String>,
    password_path: Option<PathBuf>,
) -> anyhow::Result<String> {
    let password = match (password, password_path) {
        (None, None) => prompt_password("Enter vault password: ")?,
        (Some(password), _) => password,
        (_, Some(password_path)) => read_to_string(password_path)?.trim().to_string(),
    };

    let password_generator = PasswordGenerator::from_eff_large_wordlist()?;
    Ok(password_generator.check_valid(&password)?)
}

#[cfg(test)]
mod tests {
    use crate::clap::{CliSecret, PostgresConnectionStringRedactor};

    use super::PlainRedactor;

    #[test]
    fn plain_redactor() {
        let test_value = "secret value".to_string();

        let secret_wrapper = CliSecret::<String, PlainRedactor>::new(test_value);

        let debug_value = format!("{secret_wrapper:?}");

        assert_eq!(debug_value, "<REDACTED>");
    }

    #[test]
    fn pg_redactor() {
        let test_value = "postgresql://admin:hunter2@localhost:5432/database".to_string();

        let secret_wrapper = CliSecret::<String, PostgresConnectionStringRedactor>::new(test_value);

        let debug_value = format!("{secret_wrapper:?}");

        assert_eq!(
            debug_value,
            "postgresql://admin:<REDACTED>@localhost:5432/database"
        );
    }

    #[test]
    fn pg_redactor_invalid_string() {
        let test_value = "postgres://admin:hunter2@localhost:5432/database".to_string();
        //                ^~~~~~~~ Note: not `postgresql` - so it's invalid

        let secret_wrapper = CliSecret::<String, PostgresConnectionStringRedactor>::new(test_value);

        let debug_value = format!("{secret_wrapper:?}");

        assert_eq!(
            debug_value,
            "<REDACTED IN WHOLE DUE TO UNRECOGNIZED CONNECTION STRING FORMAT>"
        );
    }
}
