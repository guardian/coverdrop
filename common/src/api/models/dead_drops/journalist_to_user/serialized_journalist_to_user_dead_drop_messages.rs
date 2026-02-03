use serde::{Deserialize, Serialize};
use serde_with::{
    base64::{Base64, Standard},
    formats::Unpadded,
    serde_as,
};
use sqlx::{error::BoxDynError, Database, Decode};

use crate::{
    api::models::messages::journalist_to_user_message::EncryptedJournalistToUserMessage,
    crypto::Signable, protocol::constants::JOURNALIST_TO_USER_ENCRYPTED_MESSAGE_LEN,
};

use super::journalist_to_user_dead_drop_messages::JournalistToUserDeadDropMessages;

/// Serialized dead drop messages. Messages are serialized into a single large block
/// of base64 in order to reduce the amount of overhead in the serialized JSON.
#[serde_as]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SerializedJournalistToUserDeadDropMessages(
    #[serde_as(as = "Base64<Standard, Unpadded>")] Vec<u8>,
);

impl SerializedJournalistToUserDeadDropMessages {
    pub fn from_vec_unchecked(vec: Vec<u8>) -> SerializedJournalistToUserDeadDropMessages {
        SerializedJournalistToUserDeadDropMessages(vec)
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn deserialize(&self) -> JournalistToUserDeadDropMessages {
        let messages = self
            .0
            .chunks(JOURNALIST_TO_USER_ENCRYPTED_MESSAGE_LEN)
            .map(|bytes| EncryptedJournalistToUserMessage::from_vec_unchecked(bytes.to_vec()))
            .collect();

        JournalistToUserDeadDropMessages { messages }
    }
}

impl Signable for SerializedJournalistToUserDeadDropMessages {
    fn as_signable_bytes(&self) -> &[u8] {
        &self.0[..]
    }
}

impl<'r, DB> Decode<'r, DB> for SerializedJournalistToUserDeadDropMessages
where
    &'r [u8]: Decode<'r, DB>,
    DB: Database,
{
    fn decode(value: DB::ValueRef<'r>) -> Result<Self, BoxDynError> {
        let value = <&[u8] as Decode<DB>>::decode(value)?;

        Ok(SerializedJournalistToUserDeadDropMessages::from_vec_unchecked(value.to_vec()))
    }
}
