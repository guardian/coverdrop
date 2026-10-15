use crate::crypto::keys::encryption::traits::PublicEncryptionKey;
use crate::crypto::keys::encryption::UnsignedEncryptionKeyPair;
use crate::crypto::{AnonymousBox, Encryptable, TwoPartyBox};
use crate::error::Error;
use crate::protocol::constants::*;
use crate::protocol::keys::{UserKeyPair, UserPublicKey};
use crate::protocol::roles::{JournalistMessaging, User};
use crate::{FixedSizeAuthenticatedMessageText, FixedSizeMessageText};
use core::fmt;
use serde::{Deserialize, Serialize};
use serde_with::{
    base64::{Base64, Standard},
    formats::Unpadded,
    serde_as,
};

/// A type alias for an [UserToJournalistMessage] that's serialized and then encrypted using
/// [AnonymousBox].
pub type EncryptedUserToJournalistMessage = AnonymousBox<SerializedUserToJournalistMessage>;

/// A type alias for a [FixedSizeAuthenticatedMessageText] that's encrypted using [TwoPartyBox].
pub type EncryptedUserToJournalistPayload = TwoPartyBox<FixedSizeAuthenticatedMessageText>;

/// Generate a new fake encrypted inner message to be used as cover using a freshly generated
/// recipient key pair. This temporary key pair is neither returned nor stored.
///
/// The result will be indistinguishable from any other [EncryptedUserToJournalistMessage]. Also, each
/// returned value will be unique due to the randomly generated key pair.
pub fn new_random_encrypted_user_to_journalist_message() -> EncryptedUserToJournalistMessage {
    let journalist_msg_key_pair = UnsignedEncryptionKeyPair::<JournalistMessaging>::generate();
    let user_key_pair = UnsignedEncryptionKeyPair::<User>::generate();

    let plaintext = FixedSizeAuthenticatedMessageText::new("").expect(
        "FixedSizeAuthenticatedMessageText::new should not fail for an empty input string.",
    );

    let message = UserToJournalistMessage::new_authenticated(
        &user_key_pair,
        journalist_msg_key_pair.public_key(),
        plaintext,
    )
    .serialize();

    AnonymousBox::encrypt(journalist_msg_key_pair.public_key(), message)
        .expect("AnonymousBox::encrypt should not fail for known valid inputs")
}

const PLACEHOLDER_RESERVED_BYTE_VALUE: u8 = 0;
const U2J_OFFSET_RESERVED_BYTE: usize = X25519_PUBLIC_KEY_LEN;

/// Once the migration of authenticated messages is fully rolled out,
/// we should be able to remove LegacyUnauthenticated messages and therefore [UserToJournalistPayload].
// TODO: Tracked in https://github.com/guardian/coverdrop-internal/issues/4240
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UserToJournalistPayload {
    Authenticated(EncryptedUserToJournalistPayload),
    LegacyUnauthenticated(FixedSizeMessageText),
}

#[derive(Clone, Eq, PartialEq)]
pub struct UserToJournalistMessage {
    pub reply_key: UserPublicKey,
    pub payload: UserToJournalistPayload,
}

impl fmt::Debug for UserToJournalistMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UserToJournalistMessage")
    }
}

impl UserToJournalistMessage {
    #[deprecated]
    pub fn new(message: FixedSizeMessageText, reply_key: impl Into<UserPublicKey>) -> Self {
        Self {
            reply_key: reply_key.into(),
            payload: UserToJournalistPayload::LegacyUnauthenticated(message),
        }
    }

    pub fn new_authenticated(
        user_key_pair: &UserKeyPair,
        journalist_msg_pk: &impl PublicEncryptionKey<JournalistMessaging>,
        message: FixedSizeAuthenticatedMessageText,
    ) -> Self {
        let payload = TwoPartyBox::encrypt(journalist_msg_pk, user_key_pair.secret_key(), message)
            .expect("encrypt should not fail");

        Self {
            reply_key: user_key_pair.public_key().into(),
            payload: UserToJournalistPayload::Authenticated(payload),
        }
    }

    pub fn serialize(&self) -> SerializedUserToJournalistMessage {
        let mut bytes = Vec::with_capacity(USER_TO_JOURNALIST_MESSAGE_LEN);
        bytes.extend(self.reply_key.as_bytes());

        match &self.payload {
            UserToJournalistPayload::Authenticated(encrypted_message) => {
                bytes.extend([AUTHENTICATED_MESSAGE_RESERVED_BYTE_VALUE]);
                bytes.extend(encrypted_message.as_bytes());
            }
            UserToJournalistPayload::LegacyUnauthenticated(message) => {
                bytes.extend([PLACEHOLDER_RESERVED_BYTE_VALUE]);
                bytes.extend(message.as_unencrypted_bytes());
            }
        }

        assert_eq!(bytes.len(), USER_TO_JOURNALIST_MESSAGE_LEN);

        SerializedUserToJournalistMessage { bytes }
    }
}

/// The serialized representation of an [UserToJournalistMessage] using the following format:
/// ```text
/// ┌────────────┬───────────────┬──────────────────┐
/// │ public_key │ reserved_byte │ payload          │
/// └────────────┴───────────────┴──────────────────┘
/// ```
/// The public key is [X25519_PUBLIC_KEY_LEN] bytes in length.
/// The reserved_byte type is [USER_TO_JOURNALIST_MESSAGE_RESERVED_BYTE_LEN] bytes in length.
/// The payload is [USER_TO_JOURNALIST_PAYLOAD_LEN] bytes in length.
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SerializedUserToJournalistMessage {
    #[serde_as(as = "Base64<Standard, Unpadded>")]
    pub bytes: Vec<u8>,
}

impl SerializedUserToJournalistMessage {
    pub fn from_vec_unchecked(bytes: Vec<u8>) -> SerializedUserToJournalistMessage {
        SerializedUserToJournalistMessage { bytes }
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn to_message(self) -> UserToJournalistMessage {
        let key_bytes = &self.bytes[..X25519_PUBLIC_KEY_LEN];
        let reply_key = UserPublicKey::from_bytes(key_bytes)
            .expect("User key should be X25519_PUBLIC_KEY_LEN bytes");

        let reserved_byte = &self.bytes[U2J_OFFSET_RESERVED_BYTE
            ..U2J_OFFSET_RESERVED_BYTE + USER_TO_JOURNALIST_MESSAGE_RESERVED_BYTE_LEN];

        let message_bytes =
            &self.bytes[U2J_OFFSET_RESERVED_BYTE + USER_TO_JOURNALIST_MESSAGE_RESERVED_BYTE_LEN..];
        let message = Vec::from(message_bytes);

        match reserved_byte {
            [AUTHENTICATED_MESSAGE_RESERVED_BYTE_VALUE] => UserToJournalistMessage {
                reply_key,
                payload: UserToJournalistPayload::Authenticated(
                    EncryptedUserToJournalistPayload::from_vec_unchecked(message),
                ),
            },
            _ => UserToJournalistMessage {
                reply_key,
                payload: UserToJournalistPayload::LegacyUnauthenticated(
                    FixedSizeMessageText::from_vec_unchecked(message),
                ),
            },
        }
    }
}

impl Encryptable for SerializedUserToJournalistMessage {
    fn as_unencrypted_bytes(&self) -> &[u8] {
        &self.bytes
    }

    fn from_unencrypted_bytes(bytes: Vec<u8>) -> Result<Self, Error> {
        Ok(Self { bytes })
    }
}

#[cfg(test)]
mod tests_user_to_journalist {
    use super::*;

    #[test]
    fn when_creating_inner_message_with_key_then_serializes_and_deserializes() -> anyhow::Result<()>
    {
        let padded_string = FixedSizeMessageText::new("test")?;
        let reply_key_pair = UnsignedEncryptionKeyPair::<User>::generate();
        #[allow(deprecated)]
        let inner_message = UserToJournalistMessage::new(
            padded_string.clone(),
            reply_key_pair.public_key().clone(),
        );

        let serialized_message = inner_message.serialize();
        let unencrypted_bytes = serialized_message.as_unencrypted_bytes();
        assert_eq!(unencrypted_bytes.len(), USER_TO_JOURNALIST_MESSAGE_LEN);

        let actual =
            SerializedUserToJournalistMessage::from_unencrypted_bytes(unencrypted_bytes.to_vec())?
                .to_message();
        assert_eq!(
            actual.payload,
            UserToJournalistPayload::LegacyUnauthenticated(padded_string)
        );
        assert_eq!(actual.reply_key, *reply_key_pair.public_key());

        Ok(())
    }

    #[test]
    fn when_creating_authenticated_message_with_key_then_serializes_and_deserializes(
    ) -> anyhow::Result<()> {
        let padded_string = FixedSizeAuthenticatedMessageText::new("test")?;
        let user_key_pair = UnsignedEncryptionKeyPair::<User>::generate();
        let journalist_key_pair = UnsignedEncryptionKeyPair::<JournalistMessaging>::generate();

        let inner_message = UserToJournalistMessage::new_authenticated(
            &user_key_pair,
            journalist_key_pair.public_key(),
            padded_string.clone(),
        );

        let serialized_message = inner_message.serialize();
        let unencrypted_bytes = serialized_message.as_unencrypted_bytes();
        assert_eq!(unencrypted_bytes.len(), USER_TO_JOURNALIST_MESSAGE_LEN);

        let actual =
            SerializedUserToJournalistMessage::from_unencrypted_bytes(unencrypted_bytes.to_vec())?
                .to_message();

        let UserToJournalistPayload::Authenticated(encrypted) = actual.payload else {
            panic!("expected an authenticated payload");
        };

        let decrypted = TwoPartyBox::decrypt(
            &actual.reply_key,
            journalist_key_pair.secret_key(),
            &encrypted,
        )?;

        assert_eq!(decrypted, padded_string);
        assert_eq!(actual.reply_key, *user_key_pair.public_key());

        Ok(())
    }

    #[test]
    fn when_creating_random_encrypted_inner_message_then_matches_length() {
        let encrypted_message = new_random_encrypted_user_to_journalist_message();
        assert_eq!(
            encrypted_message.len(),
            USER_TO_JOURNALIST_ENCRYPTED_MESSAGE_LEN
        )
    }

    #[test]
    fn when_creating_two_random_encrypted_inner_message_then_different() {
        let encrypted_message_1 = new_random_encrypted_user_to_journalist_message();
        let encrypted_message_2 = new_random_encrypted_user_to_journalist_message();
        assert_ne!(encrypted_message_1, encrypted_message_2)
    }
}
