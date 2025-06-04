use serde::{Deserialize, Serialize};

use crate::api::models::dead_drops::DeadDropId;

use super::unverified_user_to_journalist_dead_drop::UnverifiedUserToJournalistDeadDrop;

/// A list of dead drops that has been served from the API but has not yet
/// been verified against the key hierarchy.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct UnverifiedUserToJournalistDeadDropsList {
    pub dead_drops: Vec<UnverifiedUserToJournalistDeadDrop>,
}

impl UnverifiedUserToJournalistDeadDropsList {
    pub fn new(dead_drops: Vec<UnverifiedUserToJournalistDeadDrop>) -> Self {
        Self { dead_drops }
    }

    pub fn len(&self) -> usize {
        self.dead_drops.len()
    }

    pub fn is_empty(&self) -> bool {
        self.dead_drops.is_empty()
    }

    pub fn max_id(&self) -> Option<DeadDropId> {
        self.dead_drops.iter().map(|dd| dd.id).max()
    }
}
