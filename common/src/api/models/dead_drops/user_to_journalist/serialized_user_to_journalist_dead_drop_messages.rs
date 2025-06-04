use serde::{Deserialize, Serialize};
use serde_with::{
    base64::{Base64, Standard},
    formats::Unpadded,
    serde_as,
};
use sqlx::{database::HasValueRef, error::BoxDynError, Database, Decode};

use crate::{
    api::models::messages::covernode_to_journalist_message::EncryptedCoverNodeToJournalistMessage,
    protocol::constants::COVERNODE_TO_JOURNALIST_ENCRYPTED_MESSAGE_LEN,
};

use super::UserToJournalistDeadDropMessages;

/// Serialized dead drop messages. Messages are serialized into a single large block
/// of base64 in order to reduce the amount of overhead in the serialized JSON.
#[serde_as]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SerializedUserToJournalistDeadDropMessages(
    #[serde_as(as = "Base64<Standard, Unpadded>")] Vec<u8>,
);

impl SerializedUserToJournalistDeadDropMessages {
    pub fn from_vec_unchecked(vec: Vec<u8>) -> SerializedUserToJournalistDeadDropMessages {
        SerializedUserToJournalistDeadDropMessages(vec)
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn deserialize(self) -> UserToJournalistDeadDropMessages {
        let messages = self
            .0
            .chunks(COVERNODE_TO_JOURNALIST_ENCRYPTED_MESSAGE_LEN)
            .map(|bytes| EncryptedCoverNodeToJournalistMessage::from_vec_unchecked(bytes.to_vec()))
            .collect();

        UserToJournalistDeadDropMessages { messages }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0[..]
    }
}

impl<'r, DB> Decode<'r, DB> for SerializedUserToJournalistDeadDropMessages
where
    &'r [u8]: Decode<'r, DB>,
    DB: Database,
{
    fn decode(value: <DB as HasValueRef<'r>>::ValueRef) -> Result<Self, BoxDynError> {
        let value = <&[u8] as Decode<DB>>::decode(value)?;

        Ok(SerializedUserToJournalistDeadDropMessages::from_vec_unchecked(value.to_vec()))
    }
}
