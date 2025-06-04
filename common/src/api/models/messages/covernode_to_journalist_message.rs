use crate::api::models::messages::user_to_journalist_message::EncryptedUserToJournalistMessage;
use crate::crypto::keys::encryption::UnsignedEncryptionKeyPair;
use crate::crypto::{AnonymousBox, Encryptable, TwoPartyBox};
use crate::error::Error;
use crate::protocol::constants::*;
use crate::protocol::keys::CoverNodeMessagingKeyPair;
use crate::protocol::roles::JournalistMessaging;

use serde::{Deserialize, Serialize};
use serde_with::{
    base64::{Base64, Standard},
    formats::Unpadded,
    serde_as,
};

/// Generate a new fake encrypted message to be used as cover using a freshly generated
/// recipient key pair. This temporary key pair is neither returned nor stored.
///
/// The result will be indistinguishable from any other [EncryptedCoverNodeToJournalistMessage].
/// Also, each returned value will be unique due to the randomly generated key pair.
pub fn new_random_encrypted_covernode_to_journalist_message(
    covernode_msg_key_pair: &CoverNodeMessagingKeyPair,
    inner_message: EncryptedUserToJournalistMessage,
) -> anyhow::Result<EncryptedCoverNodeToJournalistMessage> {
    let random_journalist_msg_key_pair =
        UnsignedEncryptionKeyPair::<JournalistMessaging>::generate();

    let message = CoverNodeToJournalistMessage {
        payload: inner_message,
    };

    Ok(TwoPartyBox::encrypt(
        random_journalist_msg_key_pair.public_key(),
        covernode_msg_key_pair.secret_key(),
        message.serialize(),
    )?)
}

/// A type alias for an [CoverNodeToJournalistMessage] that's serialized and then encrypted using
/// [TwoPartyBox].
pub type EncryptedCoverNodeToJournalistMessage =
    TwoPartyBox<SerializedCoverNodeToJournalistMessage>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CoverNodeToJournalistMessage {
    pub payload: EncryptedUserToJournalistMessage,
}

impl CoverNodeToJournalistMessage {
    pub fn new(payload: EncryptedUserToJournalistMessage) -> Self {
        Self { payload }
    }

    pub fn serialize(&self) -> SerializedCoverNodeToJournalistMessage {
        let bytes = self.payload.as_bytes().clone();
        assert_eq!(bytes.len(), COVERNODE_TO_JOURNALIST_MESSAGE_LEN);
        SerializedCoverNodeToJournalistMessage { bytes }
    }
}

/// The serialized representation of a [CoverNodeToJournalistMessage] using the following format:
/// ```text
/// ┌───────────────────────────────────────────────┐
/// │ encrypted_inner_message                       │
/// └───────────────────────────────────────────────┘
/// ```
/// The inner message is [USER_TO_JOURNALIST_ENCRYPTED_MESSAGE_LEN] in length.
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SerializedCoverNodeToJournalistMessage {
    #[serde_as(as = "Base64<Standard, Unpadded>")]
    pub bytes: Vec<u8>,
}

impl SerializedCoverNodeToJournalistMessage {
    pub fn from_slice_unchecked(bytes: &[u8]) -> Self {
        SerializedCoverNodeToJournalistMessage {
            bytes: Vec::from(bytes),
        }
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn to_message(self) -> CoverNodeToJournalistMessage {
        CoverNodeToJournalistMessage {
            payload: AnonymousBox::from_vec_unchecked(self.bytes),
        }
    }
}

impl Encryptable for SerializedCoverNodeToJournalistMessage {
    fn as_unencrypted_bytes(&self) -> &[u8] {
        &self.bytes
    }
    fn from_unencrypted_bytes(bytes: Vec<u8>) -> Result<Self, Error> {
        Ok(Self { bytes })
    }
}
