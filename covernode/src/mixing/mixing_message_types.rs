use common::api::models::messages::journalist_to_covernode_message::JournalistToCoverNodeMessage;
use common::api::models::messages::journalist_to_user_message::{
    new_random_encrypted_journalist_to_user_message, EncryptedJournalistToUserMessage,
};
use common::api::models::messages::user_to_covernode_message::UserToCoverNodeMessage;
use common::api::models::messages::user_to_journalist_message::{
    new_random_encrypted_user_to_journalist_message, EncryptedUserToJournalistMessage,
};
use common::protocol::recipient_tag::{RecipientTag, RECIPIENT_TAG_FOR_COVER};

use sha2::{Digest, Sha256};

pub trait MixingInputMessage<OUTPUT> {
    /// Returns the inner message for real messages, otherwise `None`
    fn to_payload_if_real(self) -> Option<OUTPUT>;
}

pub trait MixingOutputMessage {
    fn generate_new_random_message() -> Self;
    fn to_hash(&self) -> MixingOutputMessageHash;
}

#[derive(Eq, Hash, PartialEq)]
pub struct MixingOutputMessageHash([u8; 32]);

impl MixingOutputMessageHash {
    pub fn new(hash: [u8; 32]) -> Self {
        Self(hash)
    }
}

//
// Implementations: user -> journalist
//

// For the direction U2J the publishing service will to know the receiving journalist to apply
// the correct keys for the outer TwoPartyBox.
type UserToJournalistMixingOutputMessage = (RecipientTag, EncryptedUserToJournalistMessage);

impl MixingInputMessage<UserToJournalistMixingOutputMessage> for UserToCoverNodeMessage {
    fn to_payload_if_real(self) -> Option<UserToJournalistMixingOutputMessage> {
        if let UserToCoverNodeMessage::Real {
            recipient_tag,
            payload,
        } = self
        {
            Some((recipient_tag, payload))
        } else {
            None
        }
    }
}

impl MixingOutputMessage for UserToJournalistMixingOutputMessage {
    fn generate_new_random_message() -> Self {
        let encrypted_message = new_random_encrypted_user_to_journalist_message();
        (RECIPIENT_TAG_FOR_COVER, encrypted_message)
    }

    fn to_hash(&self) -> MixingOutputMessageHash {
        let mut hasher = Sha256::new();
        hasher.update(self.0.as_ref());
        hasher.update(self.1.as_bytes());
        MixingOutputMessageHash::new(hasher.finalize().into())
    }
}

//
// Implementations journalist -> user
//

impl MixingInputMessage<EncryptedJournalistToUserMessage> for JournalistToCoverNodeMessage {
    fn to_payload_if_real(self) -> Option<EncryptedJournalistToUserMessage> {
        if let JournalistToCoverNodeMessage::Real { payload } = self {
            Some(payload)
        } else {
            None
        }
    }
}

impl MixingOutputMessage for EncryptedJournalistToUserMessage {
    fn generate_new_random_message() -> Self {
        new_random_encrypted_journalist_to_user_message().unwrap()
    }

    fn to_hash(&self) -> MixingOutputMessageHash {
        let mut hasher = Sha256::new();
        hasher.update(self.as_bytes());
        MixingOutputMessageHash::new(hasher.finalize().into())
    }
}
