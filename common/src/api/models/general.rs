use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(
    Clone,
    Debug,
    Deserialize,
    Eq,
    PartialEq,
    Serialize,
    clap::ValueEnum,
    sqlx::Type,
    strum::AsRefStr,
)]
#[clap(rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", deny_unknown_fields)]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum SystemStatus {
    Available,
    Unavailable,
    DegradedPerformance,
    ScheduledMaintenance,
    // Displayed when CoverDrop is started for the first time
    // and no status events are present in the database
    NoInformation,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct StatusEvent {
    pub status: SystemStatus,
    pub description: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PublishedStatusEvent {
    pub status: SystemStatus,
    pub is_available: bool,
    pub description: String,
    pub timestamp: DateTime<Utc>,
}

impl StatusEvent {
    pub fn new(status: SystemStatus, description: String, timestamp: DateTime<Utc>) -> Self {
        Self {
            status,
            description,
            timestamp,
        }
    }

    pub fn no_information(now: DateTime<Utc>) -> Self {
        Self {
            status: SystemStatus::NoInformation,
            description: "No information available".to_owned(),
            timestamp: now,
        }
    }

    pub fn into_published(self) -> PublishedStatusEvent {
        let is_available = matches!(
            self.status,
            SystemStatus::Available | SystemStatus::DegradedPerformance
        );

        PublishedStatusEvent {
            status: self.status,
            is_available,
            description: self.description,
            timestamp: self.timestamp,
        }
    }
}
