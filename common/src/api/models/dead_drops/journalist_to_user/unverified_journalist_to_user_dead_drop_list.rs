use serde::{Deserialize, Serialize};

use crate::api::models::dead_drops::DeadDropId;

use super::UnverifiedJournalistToUserDeadDrop;

/// A list of dead drops that has been served from the API but has not yet
/// been verified against the key hierarchy.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct UnverifiedJournalistToUserDeadDropsList {
    pub dead_drops: Vec<UnverifiedJournalistToUserDeadDrop>,
}

impl UnverifiedJournalistToUserDeadDropsList {
    pub fn new(dead_drops: Vec<UnverifiedJournalistToUserDeadDrop>) -> Self {
        Self { dead_drops }
    }

    pub fn len(&self) -> usize {
        self.dead_drops.len()
    }

    pub fn is_empty(&self) -> bool {
        self.dead_drops.is_empty()
    }

    pub fn max_id(&self) -> Option<DeadDropId> {
        self.dead_drops.iter().map(|dead_drop| dead_drop.id).max()
    }
}
