use crate::api::models::journalist_id::{JournalistIdentity, MAX_JOURNALIST_IDENTITY_LEN};
use crate::api::models::messages::{FLAG_J2U_MESSAGE_TYPE_HANDOVER, FLAG_J2U_MESSAGE_TYPE_MESSAGE};
use crate::crypto::keys::encryption::UnsignedEncryptionKeyPair;
use crate::crypto::{Encryptable, TwoPartyBox};
use crate::protocol::constants::JOURNALIST_TO_USER_MESSAGE_LEN;
use crate::protocol::roles::{JournalistMessaging, User};
use crate::{FixedSizeMessageText, PaddedCompressedString};
use core::fmt;

use crate::error::Error;
use serde::{Deserialize, Serialize};
use serde_with::{
    base64::{Base64, Standard},
    formats::Unpadded,
    serde_as,
};

/// A type alias for an [JournalistToUserMessage] that's serialized and then encrypted using
/// [AnonymousBox].
pub type EncryptedJournalistToUserMessage = TwoPartyBox<SerializedJournalistToUserMessage>;

/// Generate a new fake encrypted inner message to be used as cover using a freshly generated
/// recipient key pair. This temporary key pair is neither returned nor stored.
///
/// The result will be indistinguishable from any other [EncryptedJournalistToUserMessage]. Also, each
/// returned value will be unique due to the randomly generated key pair.
pub fn new_random_encrypted_journalist_to_user_message(
) -> anyhow::Result<EncryptedJournalistToUserMessage> {
    let journalist_msg_key_pair = UnsignedEncryptionKeyPair::<JournalistMessaging>::generate();
    let user_key_pair = UnsignedEncryptionKeyPair::<User>::generate();

    let plaintext = FixedSizeMessageText::new("")?;
    let message = JournalistToUserMessage::new_with_message(plaintext).serialize();

    Ok(TwoPartyBox::encrypt(
        journalist_msg_key_pair.public_key(),
        user_key_pair.secret_key(),
        message,
    )?)
}

#[derive(Clone, Eq, PartialEq)]
pub enum JournalistToUserMessage {
    Message(FixedSizeMessageText),
    HandOver(JournalistIdentity),
}

impl fmt::Debug for JournalistToUserMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JournalistToUserMessage")
    }
}

impl JournalistToUserMessage {
    pub fn new_with_message(message: FixedSizeMessageText) -> Self {
        Self::Message(message)
    }

    pub fn new_with_hand_over(journalist_id: JournalistIdentity) -> Self {
        // Repeat an assertion made in the journalist identity creation function
        // to be extra careful of refactoring
        assert!(journalist_id.len() < MAX_JOURNALIST_IDENTITY_LEN);
        Self::HandOver(journalist_id)
    }

    pub fn get_type_flag(&self) -> u8 {
        match self {
            JournalistToUserMessage::Message(_) => FLAG_J2U_MESSAGE_TYPE_MESSAGE,
            JournalistToUserMessage::HandOver(_) => FLAG_J2U_MESSAGE_TYPE_HANDOVER,
        }
    }

    pub fn serialize(&self) -> SerializedJournalistToUserMessage {
        let mut bytes = Vec::with_capacity(JOURNALIST_TO_USER_MESSAGE_LEN);

        let type_flag = self.get_type_flag();
        bytes.push(type_flag);

        match self {
            Self::Message(message) => {
                bytes.extend(message.as_unencrypted_bytes());
            }
            Self::HandOver(journalist_id) => {
                bytes.extend(journalist_id.as_bytes());
                bytes.resize(JOURNALIST_TO_USER_MESSAGE_LEN, b'\0');
            }
        }

        assert_eq!(bytes.len(), JOURNALIST_TO_USER_MESSAGE_LEN);

        SerializedJournalistToUserMessage { bytes }
    }
}

/// The serialized representation of an [JournalistToUserMessage] using the following format:
/// ```text
/// ┌──────────────────────────────────┐
/// │ padded_message                   │
/// └──────────────────────────────────┘
/// ```
/// The padded message is [JOURNALIST_TO_USER_PADDED_MESSAGE_LEN] in length.
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SerializedJournalistToUserMessage {
    #[serde_as(as = "Base64<Standard, Unpadded>")]
    pub bytes: Vec<u8>,
}

impl SerializedJournalistToUserMessage {
    pub fn from_vec_unchecked(bytes: Vec<u8>) -> SerializedJournalistToUserMessage {
        SerializedJournalistToUserMessage { bytes }
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn to_message(self) -> anyhow::Result<JournalistToUserMessage> {
        let message_bytes = self.bytes;
        let message_type = message_bytes[0];
        let content = &message_bytes[1..];

        match message_type {
            FLAG_J2U_MESSAGE_TYPE_MESSAGE => Ok(JournalistToUserMessage::new_with_message(
                PaddedCompressedString::from_vec_unchecked(content.to_vec()),
            )),
            FLAG_J2U_MESSAGE_TYPE_HANDOVER => {
                let Some(end) = content.iter().position(|&c| c == b'\0') else {
                    anyhow::bail!("Could not end of journalist identity string")
                };

                let journalist_id = String::from_utf8(content[..end].to_vec())?;
                let journalist_id = JournalistIdentity::new(&journalist_id)?;

                Ok(JournalistToUserMessage::new_with_hand_over(journalist_id))
            }
            _ => anyhow::bail!(
                "Serialized journalist to user message does not have a valid type flag"
            ),
        }
    }
}

impl Encryptable for SerializedJournalistToUserMessage {
    fn as_unencrypted_bytes(&self) -> &[u8] {
        &self.bytes
    }

    fn from_unencrypted_bytes(bytes: Vec<u8>) -> Result<Self, Error> {
        Ok(Self { bytes })
    }
}

#[cfg(test)]
mod tests_journalist_to_user {
    use super::*;
    use crate::protocol::constants::JOURNALIST_TO_USER_ENCRYPTED_MESSAGE_LEN;

    #[test]
    fn all_j2u_message_types_round_trip() -> anyhow::Result<()> {
        let messages = vec![
            JournalistToUserMessage::new_with_message(FixedSizeMessageText::new("test")?),
            JournalistToUserMessage::new_with_hand_over(JournalistIdentity::new("id")?),
        ];

        for message in messages {
            let serialized_message = message.serialize();

            assert_eq!(serialized_message.len(), JOURNALIST_TO_USER_MESSAGE_LEN);

            let deserialized_message = serialized_message.to_message()?;

            assert_eq!(message, deserialized_message);
        }

        Ok(())
    }

    #[test]
    fn when_creating_random_encrypted_inner_message_then_matches_length() -> anyhow::Result<()> {
        let encrypted_message = new_random_encrypted_journalist_to_user_message()?;
        assert_eq!(
            encrypted_message.len(),
            JOURNALIST_TO_USER_ENCRYPTED_MESSAGE_LEN
        );
        Ok(())
    }

    #[test]
    fn when_creating_two_random_encrypted_inner_message_then_different() -> anyhow::Result<()> {
        let encrypted_message_1 = new_random_encrypted_journalist_to_user_message()?;
        let encrypted_message_2 = new_random_encrypted_journalist_to_user_message()?;
        assert_ne!(encrypted_message_1, encrypted_message_2);
        Ok(())
    }
}
