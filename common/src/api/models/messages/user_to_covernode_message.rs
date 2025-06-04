use crate::api::models::messages::user_to_journalist_message::EncryptedUserToJournalistMessage;
use crate::crypto::{AnonymousBox, Encryptable, MultiAnonymousBox};
use crate::error::Error;
use crate::protocol::constants::*;
use crate::protocol::recipient_tag::{RecipientTag, RECIPIENT_TAG_FOR_COVER};
use serde::{Deserialize, Serialize};
use serde_with::{
    base64::{Base64, Standard},
    formats::Unpadded,
    serde_as,
};

pub type EncryptedUserToCoverNodeMessage =
    MultiAnonymousBox<SerializedUserToCoverNodeMessage, COVERNODE_WRAPPING_KEY_COUNT>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum UserToCoverNodeMessage {
    Real {
        recipient_tag: RecipientTag,
        payload: EncryptedUserToJournalistMessage,
    },
    Cover,
}

impl UserToCoverNodeMessage {
    pub fn new_real_message(
        recipient_tag: RecipientTag,
        payload: EncryptedUserToJournalistMessage,
    ) -> Self {
        Self::Real {
            recipient_tag,
            payload,
        }
    }

    pub fn new_cover_message() -> Self {
        Self::Cover
    }

    pub fn serialize(&self) -> SerializedUserToCoverNodeMessage {
        let mut bytes = Vec::with_capacity(USER_TO_COVERNODE_MESSAGE_LEN);

        match self {
            Self::Real {
                recipient_tag,
                payload,
            } => {
                bytes.extend(recipient_tag.as_ref());
                bytes.extend(payload.as_ref());
            }
            Self::Cover => {
                bytes.extend(RECIPIENT_TAG_FOR_COVER.as_ref());
                let cover_bytes = [0_u8; USER_TO_JOURNALIST_ENCRYPTED_MESSAGE_LEN];
                bytes.extend(cover_bytes);
            }
        }

        assert_eq!(bytes.len(), USER_TO_COVERNODE_MESSAGE_LEN);

        SerializedUserToCoverNodeMessage { bytes }
    }
}

/// The serialized representation of a [UserToCoverNodeMessage] using the following format:
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
pub struct SerializedUserToCoverNodeMessage {
    #[serde_as(as = "Base64<Standard, Unpadded>")]
    pub bytes: Vec<u8>,
}

impl SerializedUserToCoverNodeMessage {
    pub fn from_slice_unchecked(bytes: &[u8]) -> Self {
        SerializedUserToCoverNodeMessage {
            bytes: Vec::from(bytes),
        }
    }

    // Ignoring the clippy `len_without_is_empty` since this isn't a container
    // possibly `len` should be renamed.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn to_message(self) -> UserToCoverNodeMessage {
        let recipient_tag =
            RecipientTag::from_bytes(&self.bytes[..RECIPIENT_TAG_LEN]).expect("bad recipient tag");

        if recipient_tag == RECIPIENT_TAG_FOR_COVER {
            UserToCoverNodeMessage::Cover
        } else {
            let message_bytes = &self.bytes[RECIPIENT_TAG_LEN..];
            let vec = Vec::from(message_bytes);

            UserToCoverNodeMessage::Real {
                recipient_tag,
                payload: AnonymousBox::from_vec_unchecked(vec),
            }
        }
    }
}

impl Encryptable for SerializedUserToCoverNodeMessage {
    fn as_unencrypted_bytes(&self) -> &[u8] {
        &self.bytes
    }

    fn from_unencrypted_bytes(bytes: Vec<u8>) -> Result<Self, Error> {
        Ok(Self { bytes })
    }
}
