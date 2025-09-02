use crate::{
    api::models::messages::journalist_to_user_message::EncryptedJournalistToUserMessage,
    protocol::constants::JOURNALIST_TO_USER_ENCRYPTED_MESSAGE_LEN,
};

use super::SerializedJournalistToUserDeadDropMessages;

/// Deserialized dead drop messages
#[derive(Debug, PartialEq, Eq)]
pub struct JournalistToUserDeadDropMessages {
    pub messages: Vec<EncryptedJournalistToUserMessage>,
}

impl JournalistToUserDeadDropMessages {
    pub fn new(
        messages: Vec<EncryptedJournalistToUserMessage>,
    ) -> JournalistToUserDeadDropMessages {
        JournalistToUserDeadDropMessages { messages }
    }

    pub fn serialize(&self) -> SerializedJournalistToUserDeadDropMessages {
        let mut buf =
            Vec::with_capacity(self.messages.len() * JOURNALIST_TO_USER_ENCRYPTED_MESSAGE_LEN);

        for message in self.messages.iter() {
            buf.extend_from_slice(message.as_bytes());
        }

        SerializedJournalistToUserDeadDropMessages::from_vec_unchecked(buf)
    }
}
