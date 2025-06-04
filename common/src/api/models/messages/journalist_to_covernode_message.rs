use crate::api::models::messages::journalist_to_user_message::EncryptedJournalistToUserMessage;
use crate::api::models::messages::{FLAG_J2U_COVER, FLAG_J2U_REAL};
use crate::crypto::{Encryptable, MultiAnonymousBox, TwoPartyBox};
use crate::error::Error;
use crate::protocol::constants::*;
use serde::{Deserialize, Serialize};
use serde_with::{
    base64::{Base64, Standard},
    formats::Unpadded,
    serde_as,
};

/// A type alias for an [JournalistToCoverNodeMessage] that's serialized and then encrypted using
/// [AnonymousBox].
pub type EncryptedJournalistToCoverNodeMessage =
    MultiAnonymousBox<SerializedJournalistToCoverNodeMessage, COVERNODE_WRAPPING_KEY_COUNT>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum JournalistToCoverNodeMessage {
    Real {
        payload: EncryptedJournalistToUserMessage,
    },
    Cover,
}

impl JournalistToCoverNodeMessage {
    pub fn new_real_message(payload: EncryptedJournalistToUserMessage) -> Self {
        Self::Real { payload }
    }

    pub fn new_cover_message() -> Self {
        Self::Cover
    }

    pub fn serialize(&self) -> SerializedJournalistToCoverNodeMessage {
        let mut bytes = Vec::with_capacity(JOURNALIST_TO_COVERNODE_MESSAGE_LEN);

        match self {
            Self::Real { payload } => {
                bytes.push(FLAG_J2U_REAL);
                bytes.extend(payload.as_ref());
            }
            Self::Cover => {
                bytes.push(FLAG_J2U_COVER);
                let cover_bytes = [0_u8; JOURNALIST_TO_USER_ENCRYPTED_MESSAGE_LEN];
                bytes.extend(cover_bytes);
            }
        }

        assert_eq!(bytes.len(), JOURNALIST_TO_COVERNODE_MESSAGE_LEN);

        SerializedJournalistToCoverNodeMessage { bytes }
    }
}

/// The serialized representation of a [JournalistToCoverNodeMessage] using the following format:
/// ```text
/// ┌─────┬───────────────────────────────────────────────┐
/// │flag │ encrypted_inner_covernode_message             │
/// └─────┴───────────────────────────────────────────────┘
/// ```
/// The flag is 1 byte in length.
/// The inner message is [COVERNODE_ENCRYPTED_INNER_MESSAGE_LEN] in length.
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SerializedJournalistToCoverNodeMessage {
    #[serde_as(as = "Base64<Standard, Unpadded>")]
    pub bytes: Vec<u8>,
}

impl SerializedJournalistToCoverNodeMessage {
    pub fn from_slice_unchecked(bytes: &[u8]) -> Self {
        SerializedJournalistToCoverNodeMessage {
            bytes: Vec::from(bytes),
        }
    }

    // Ignoring the clippy `len_without_is_empty` since this isn't a container
    // possibly `len` should be renamed.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn to_message(self) -> JournalistToCoverNodeMessage {
        let is_real = self.bytes[0] == FLAG_J2U_REAL;

        if is_real {
            let message_bytes = &self.bytes[1..];
            let vec = Vec::from(message_bytes);

            JournalistToCoverNodeMessage::Real {
                payload: TwoPartyBox::from_vec_unchecked(vec),
            }
        } else {
            JournalistToCoverNodeMessage::Cover
        }
    }
}

impl Encryptable for SerializedJournalistToCoverNodeMessage {
    fn as_unencrypted_bytes(&self) -> &[u8] {
        &self.bytes
    }

    fn from_unencrypted_bytes(bytes: Vec<u8>) -> Result<Self, Error> {
        Ok(Self { bytes })
    }
}
