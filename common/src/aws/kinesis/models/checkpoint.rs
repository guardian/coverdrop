use std::{collections::HashMap, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::api::models::messages::{
    journalist_to_covernode_message::EncryptedJournalistToCoverNodeMessage,
    user_to_covernode_message::EncryptedUserToCoverNodeMessage,
};

#[derive(Clone, Debug)]
pub struct EncryptedJournalistToCoverNodeMessageWithCheckpointsJson {
    pub message: EncryptedJournalistToCoverNodeMessage,
    pub checkpoints_json: CheckpointsJson,
}

#[derive(Clone, Debug)]
pub struct EncryptedUserToCoverNodeMessageWithCheckpointsJson {
    pub message: EncryptedUserToCoverNodeMessage,
    pub checkpoints_json: CheckpointsJson,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(transparent, deny_unknown_fields)]
pub struct SequenceNumber(String);

impl From<String> for SequenceNumber {
    fn from(value: String) -> Self {
        SequenceNumber(value)
    }
}

impl From<&str> for SequenceNumber {
    fn from(value: &str) -> Self {
        SequenceNumber(value.to_owned())
    }
}

impl Display for SequenceNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent, deny_unknown_fields)]
pub struct Checkpoints(HashMap<String, SequenceNumber>);

impl Checkpoints {
    pub fn new() -> Self {
        Checkpoints(HashMap::new())
    }

    pub fn get(&self, k: &str) -> Option<&SequenceNumber> {
        self.0.get(k)
    }

    pub fn insert(&mut self, k: String, v: SequenceNumber) {
        self.0.insert(k, v);
    }
}

impl Default for Checkpoints {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct CheckpointsJson(String);

impl CheckpointsJson {
    pub fn new(checkpoints: &Checkpoints) -> Result<Self, serde_json::Error> {
        let json = serde_json::to_string(checkpoints)?;
        Ok(CheckpointsJson(json))
    }
}

impl AsRef<[u8]> for CheckpointsJson {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

#[derive(Clone)]
pub struct StoredCheckpoints {
    pub user_to_journalist_checkpoints: Checkpoints,
    pub journalist_to_user_checkpoints: Checkpoints,
}
