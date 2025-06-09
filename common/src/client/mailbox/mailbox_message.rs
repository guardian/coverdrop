use std::io;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Decode, Encode};

use crate::api::models::journalist_id::JournalistIdentity;
use crate::protocol::keys::UserPublicKey;
use crate::read_ext::ReadExt;
use crate::{cover_serializable::CoverSerializable, crypto::Encryptable, FixedSizeMessageText};

use super::{message_sender::MessageSender, message_timestamp::MessageTimestamp};

#[derive(
    Clone,
    Debug,
    Eq,
    Serialize,
    Deserialize,
    Decode,
    Encode,
    strum::Display,
    strum::EnumString,
    PartialEq,
)]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum UserStatus {
    Active,
    Muted,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MailboxMessage {
    // FIXME: https://github.com/guardian/coverdrop/issues/2649
    pub id: i64,
    pub to: MessageSender,
    pub from: MessageSender,
    // Reuse the `FixedSizeMessageText` which is used when sending and receiving a message.
    // This format is fixed size which means we can easily generate a cover version which is
    // helpful in the user's mailbox, which must be a fixed size.
    pub message: FixedSizeMessageText,
    pub received_at: MessageTimestamp,
    pub read: bool,
    pub is_sent: bool,
    // TODO should this be two different type?
    pub user_status: Option<UserStatus>,
}

impl MailboxMessage {
    pub fn from_user_to_journalist(
        id: i64,
        to: &JournalistIdentity,
        from: &UserPublicKey,
        message: &FixedSizeMessageText,
        received_at: DateTime<Utc>,
        read: bool,
        user_status: Option<UserStatus>,
    ) -> Self {
        Self {
            id,
            to: MessageSender::from_journalist_id(to),
            from: MessageSender::from_user_pk(from),
            message: message.clone(),
            received_at: MessageTimestamp::new(received_at),
            read,
            is_sent: true,
            user_status,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_journalist_to_user(
        id: i64,
        to: &UserPublicKey,
        from: &JournalistIdentity,
        message: &FixedSizeMessageText,
        received_at: DateTime<Utc>,
        read: bool,
        is_sent: bool,
        user_status: Option<UserStatus>,
    ) -> Self {
        Self {
            id,
            to: MessageSender::from_user_pk(to),
            from: MessageSender::from_journalist_id(from),
            message: message.clone(),
            received_at: MessageTimestamp::new(received_at),
            read,
            is_sent,
            user_status,
        }
    }

    pub fn is_from_user(&self) -> bool {
        matches!(self.from, MessageSender::User(_))
    }

    pub fn is_journalist(&self) -> bool {
        matches!(self.from, MessageSender::Journalist(_))
    }
}

const IS_COVER_MESSAGE: u8 = 0x0;
const IS_REAL_MESSAGE: u8 = 0x1;

impl CoverSerializable for MailboxMessage {
    fn read<R>(reader: &mut R) -> anyhow::Result<Option<Self>>
    where
        R: ReadExt + io::Seek,
    {
        let is_real = reader.read_byte()?;

        if is_real == IS_REAL_MESSAGE {
            let to = MessageSender::read(reader)?;
            let from = MessageSender::read(reader)?;

            let mut message_bytes = [0; FixedSizeMessageText::TOTAL_LEN];
            reader.read_exact(&mut message_bytes)?;
            let message_bytes = Vec::from(message_bytes);
            let message = FixedSizeMessageText::from_unencrypted_bytes(message_bytes)?;

            let received_at = MessageTimestamp::read(reader)?;

            Ok(Some(MailboxMessage {
                // User mailbox messages don't have IDs:
                // See https://github.com/guardian/coverdrop/issues/2649
                id: 0,
                to,
                from,
                message,
                received_at,
                // Not tracking read status when reading out of a user mailbox
                // assume all messages have been read
                read: true,
                is_sent: false,
                user_status: None,
            }))
        } else {
            let to_size = MessageSender::SERIALIZED_LEN;
            let from_size = MessageSender::SERIALIZED_LEN;
            let message_size = FixedSizeMessageText::TOTAL_LEN;
            let received_at_size = MessageTimestamp::SERIALIZED_LEN;

            let skip = i64::try_from(to_size + from_size + message_size + received_at_size)
                .expect("Serialized sizes valid");

            reader.seek(std::io::SeekFrom::Current(skip))?;
            Ok(None)
        }
    }

    const SERIALIZED_LEN: usize = 1 // is_real/is_cover byte
        + MessageSender::SERIALIZED_LEN // to
        + MessageSender::SERIALIZED_LEN // from
        + FixedSizeMessageText::TOTAL_LEN // message
        + MessageTimestamp::SERIALIZED_LEN; // timestamp

    fn unchecked_write_real(&self, writer: &mut impl io::Write) -> anyhow::Result<()> {
        writer.write_all(&[IS_REAL_MESSAGE])?;

        self.to.write(writer)?;
        self.from.write(writer)?;
        writer.write_all(self.message.as_unencrypted_bytes())?;
        self.received_at.write(writer)?;

        Ok(())
    }

    fn unchecked_write_cover(writer: &mut impl io::Write) -> anyhow::Result<()> {
        writer.write_all(&[IS_COVER_MESSAGE])?;

        let cover_sender = MessageSender::default();
        cover_sender.write(writer)?; // to
        cover_sender.write(writer)?; // from

        writer.write_all(&[0; FixedSizeMessageText::TOTAL_LEN])?;
        writer.write_all(&[0; MessageTimestamp::SERIALIZED_LEN])?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::{
        crypto::keys::encryption::UnsignedEncryptionKeyPair, protocol::roles::User, time,
        FixedSizeMessageText,
    };

    use super::*;

    #[test]
    fn can_roundtrip() -> anyhow::Result<()> {
        let journalist_id = JournalistIdentity::new("journalist")?;

        let message = FixedSizeMessageText::new("text")?;
        let user_key_pair = UnsignedEncryptionKeyPair::<User>::generate();

        let now = time::now();

        let msg_1 = MailboxMessage::from_journalist_to_user(
            0,
            user_key_pair.public_key(),
            &journalist_id,
            &message,
            now,
            true,
            false,
            None,
        );

        let mut buf = Cursor::new(vec![]);

        msg_1.write_real(&mut buf)?;

        // Emulate writing to disk and then reading it off again
        buf.set_position(0);

        let msg_2 = MailboxMessage::read(&mut buf)?.expect("Read real mailbox message");

        assert_eq!(msg_1, msg_2);

        Ok(())
    }

    #[test]
    fn cover_and_real_same_size() -> anyhow::Result<()> {
        let journalist_id = JournalistIdentity::new("journalist")?;
        let message = FixedSizeMessageText::new("text")?;

        let user_key_pair = UnsignedEncryptionKeyPair::<User>::generate();

        let now = time::now();

        let user_mailbox_message = MailboxMessage::from_user_to_journalist(
            0,
            &journalist_id,
            user_key_pair.public_key(),
            &message,
            now,
            false,
            None,
        );

        let journalist_vault_message = MailboxMessage::from_journalist_to_user(
            1,
            user_key_pair.public_key(),
            &journalist_id,
            &message,
            now,
            false,
            false,
            None,
        );

        let mut user_real_buf = Cursor::new(vec![]);
        let mut journalist_real_buf = Cursor::new(vec![]);
        let mut cover_buf = Cursor::new(vec![]);

        user_mailbox_message.write_real(&mut user_real_buf)?;
        journalist_vault_message.write_real(&mut journalist_real_buf)?;
        MailboxMessage::write_cover(&mut cover_buf)?;

        let user_len = user_real_buf.into_inner().len();
        let journalist_len = journalist_real_buf.into_inner().len();
        let cover_len = cover_buf.into_inner().len();

        assert_eq!(user_len, cover_len);
        assert_eq!(user_len, journalist_len);

        Ok(())
    }
}
