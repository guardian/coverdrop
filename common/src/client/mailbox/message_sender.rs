use std::fmt::{Display, Formatter};
use std::io;

use serde::{Deserialize, Serialize};

use crate::api::models::journalist_id::JournalistIdentity;
use crate::protocol::constants::{RECIPIENT_TAG_LEN, X25519_PUBLIC_KEY_LEN};
use crate::protocol::keys::UserPublicKey;
use crate::protocol::recipient_tag::RecipientTag;
use crate::read_ext::ReadExt;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum MessageSender {
    /// This message has been sent by a user
    User([u8; X25519_PUBLIC_KEY_LEN]),
    /// This message is sent by the journalist who controls this mailbox
    Journalist(RecipientTag),
}

const USER_BYTE: u8 = 0x1;
const JOURNALIST_BYTE: u8 = 0x2;

impl MessageSender {
    pub const SERIALIZED_LEN: usize = X25519_PUBLIC_KEY_LEN + 1;

    pub fn from_user_pk(pk: &UserPublicKey) -> Self {
        Self::User(*pk.key.as_bytes())
    }

    pub fn from_journalist_id(journalist_id: &JournalistIdentity) -> Self {
        let tag = RecipientTag::from_journalist_id(journalist_id);
        Self::Journalist(tag)
    }

    pub fn as_bytes(&self) -> &[u8] {
        match self {
            MessageSender::User(key) => &key[..],
            MessageSender::Journalist(tag) => tag.as_ref(),
        }
    }

    pub fn to_user_pk(&self) -> anyhow::Result<Option<UserPublicKey>> {
        match self {
            MessageSender::User(key) => Ok(Some(UserPublicKey::from_bytes(key)?)),
            MessageSender::Journalist(_) => Ok(None),
        }
    }

    /// Get the reply key for a message sender as a hex string if it is a user.
    /// Returns `None` if the message sender is the journalist.
    pub fn to_user_reply_key_hex(&self) -> Option<String> {
        match self {
            MessageSender::User(key) => Some(hex::encode(key)),
            MessageSender::Journalist(_) => None,
        }
    }

    pub fn to_journalist_tag(&self) -> Option<RecipientTag> {
        match self {
            MessageSender::User(_) => None,
            MessageSender::Journalist(tag) => Some(tag.clone()),
        }
    }

    pub fn read<R>(reader: &mut R) -> anyhow::Result<Self>
    where
        R: ReadExt + io::Seek,
    {
        let type_flag = reader.read_byte()?;

        let mut key_buf = [0; X25519_PUBLIC_KEY_LEN];
        reader.read_exact(&mut key_buf)?;

        let sender = match type_flag {
            USER_BYTE => Self::User(key_buf),
            JOURNALIST_BYTE => {
                Self::Journalist(RecipientTag::from_bytes(&key_buf[..RECIPIENT_TAG_LEN])?)
            }
            _ => panic!("Unrecognised flag in message sender field"),
        };

        Ok(sender)
    }

    pub fn write(&self, writer: &mut impl io::Write) -> anyhow::Result<()> {
        match self {
            MessageSender::User(key) => {
                writer.write_all(&[USER_BYTE])?;
                writer.write_all(&key[..])?;
            }
            MessageSender::Journalist(tag) => {
                let mut padded_bytes = [0_u8; X25519_PUBLIC_KEY_LEN];

                padded_bytes[..RECIPIENT_TAG_LEN].copy_from_slice(tag.as_ref());

                writer.write_all(&[JOURNALIST_BYTE])?;
                writer.write_all(&padded_bytes)?;
            }
        }

        Ok(())
    }
}

impl Default for MessageSender {
    fn default() -> Self {
        Self::User([0; X25519_PUBLIC_KEY_LEN])
    }
}

impl Display for MessageSender {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageSender::User(key) => {
                write!(f, "User({}...)", hex::encode(&key[..8]))
            }
            MessageSender::Journalist(tag) => {
                write!(f, "Journalist({}...)", hex::encode(tag))
            }
        }
    }
}
