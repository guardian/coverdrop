use crate::{
    api::models::messages::covernode_to_journalist_message::EncryptedCoverNodeToJournalistMessage,
    protocol::constants::COVERNODE_TO_JOURNALIST_ENCRYPTED_MESSAGE_LEN,
};

use super::SerializedUserToJournalistDeadDropMessages;

/// Deserialized user to journalist dead drop messages. No longer
/// a single large blob of binary data, messages can be accessed individually.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserToJournalistDeadDropMessages {
    // When going from U2J the CoverNode will re-encrypt the message
    // to prevent N-1 attacks. This is why the messages are not actually
    // U2J, but for the sake of simplicity in naming we call the struct
    // UserToJournalist...
    pub messages: Vec<EncryptedCoverNodeToJournalistMessage>,
}

impl UserToJournalistDeadDropMessages {
    pub fn new(
        messages: Vec<EncryptedCoverNodeToJournalistMessage>,
    ) -> UserToJournalistDeadDropMessages {
        UserToJournalistDeadDropMessages { messages }
    }

    pub fn serialize(&self) -> SerializedUserToJournalistDeadDropMessages {
        let mut buf =
            Vec::with_capacity(self.messages.len() * COVERNODE_TO_JOURNALIST_ENCRYPTED_MESSAGE_LEN);
        // Deserialization works because all messages have a known length
        for message in self.messages.iter() {
            buf.extend_from_slice(message.as_bytes());
        }

        SerializedUserToJournalistDeadDropMessages::from_vec_unchecked(buf)
    }
}
