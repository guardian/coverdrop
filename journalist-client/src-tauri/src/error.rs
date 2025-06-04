use serde::{Serialize, Serializer};
use snafu::{prelude::*, Location};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum CommandError {
    // These three errors all imply that the user has not unlocked their
    // vault correctly
    #[snafu(display("API Client not available"))]
    ApiClientUnavailable,
    #[snafu(display("Vault is locked"))]
    VaultLocked,
    #[snafu(display("Public keys and profiles not available"))]
    PublicInfoUnavailable,
    #[snafu(display("Could not find profile"))]
    MissingProfile,
    #[snafu(display("Failed to serialize JSON"))]
    JsonSerialize {
        source: serde_json::Error,
        #[snafu(implicit)]
        location: Location,
    },
    #[snafu(display("Vault failed to {failed_to}"))]
    Vault {
        failed_to: &'static str,
        source: anyhow::Error,
        #[snafu(implicit)]
        location: Location,
    },
    #[snafu(display("{source:?}"))]
    Common {
        source: common::Error,
        #[snafu(implicit)]
        location: Location,
    },
    // Generic error from an anyhow source where you'd like the
    // source printed
    #[snafu(display("Failed to {failed_to}"))]
    Anyhow {
        failed_to: &'static str,
        source: anyhow::Error,
        #[snafu(implicit)]
        location: Location,
    },
    #[snafu(display("Failed to {failed_to}"))]
    Io {
        failed_to: &'static str,
        source: std::io::Error,
        #[snafu(implicit)]
        location: Location,
    },
    // A generic error where we just want to print a string
    // to the user. This does not log any source errors!
    #[snafu(display("{ctx}"))]
    Generic {
        ctx: &'static str,
        #[snafu(implicit)]
        location: Location,
    },
}

impl Serialize for CommandError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Any time we're serializing a command error it's because
        // something has failed so we should log that
        tracing::error!("Command error: {:?}", self);
        serializer.serialize_str(&self.to_string())
    }
}
