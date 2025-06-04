use std::{
    io::{Cursor, Read, Seek, Write},
    mem::size_of,
};

use crate::{
    api::models::dead_drops::DeadDropId,
    client::mailbox::mailbox_message::MailboxMessage,
    crypto::{
        keys::{
            encryption::{traits::PublicEncryptionKey, UnsignedEncryptionKeyPair},
            X25519PublicKey, X25519SecretKey,
        },
        SecretBox, SecretBoxKey, SECRET_BOX_FOOTER_LEN,
    },
    protocol::{
        constants::{X25519_PUBLIC_KEY_LEN, X25519_SECRET_KEY_LEN},
        keys::UserKeyPair,
    },
    FixedBuffer,
};

use super::MAX_MAILBOX_MESSAGES;

pub type FixedMessageBuffer = FixedBuffer<MailboxMessage, MAX_MAILBOX_MESSAGES>;

#[derive(Clone)]
pub struct SecretMailboxData {
    pub user_key_pair: UserKeyPair,
    pub max_dead_drop_id: DeadDropId,
    pub messages: FixedMessageBuffer,
}

impl SecretMailboxData {
    pub const SERIALIZED_LEN: usize = SECRET_BOX_FOOTER_LEN
        + X25519_PUBLIC_KEY_LEN // User public key
        + X25519_SECRET_KEY_LEN // User secret key
        + FixedMessageBuffer::SERIALIZED_LEN  // Mailbox messages
        + size_of::<DeadDropId>(); // Next dead drop ID

    /// Deserialize secret data from a vector containing *only* the encrypted secret mailbox data.
    pub fn deserialize(
        encrypted_data: Vec<u8>,
        key: &SecretBoxKey,
    ) -> anyhow::Result<SecretMailboxData> {
        let ciphertext = SecretBox::<Vec<u8>>::from_vec_unchecked(encrypted_data);
        let plaintext = SecretBox::decrypt(key, ciphertext)?;

        let mut cursor = Cursor::new(plaintext);

        let mut user_pk_buf = [0; X25519_PUBLIC_KEY_LEN];
        cursor.read_exact(&mut user_pk_buf)?;
        let user_pk = X25519PublicKey::from(user_pk_buf);

        let mut user_sk_buf = [0; X25519_SECRET_KEY_LEN];
        cursor.read_exact(&mut user_sk_buf)?;
        let user_sk = X25519SecretKey::from(user_sk_buf);

        let user_key_pair = UnsignedEncryptionKeyPair::from_raw_keys(user_pk, user_sk);

        let messages = FixedMessageBuffer::read(&mut cursor)?;

        let mut max_dead_drop_id_buf = [0; size_of::<DeadDropId>()];
        cursor.read_exact(&mut max_dead_drop_id_buf)?;
        let max_dead_drop_id = DeadDropId::from_be_bytes(max_dead_drop_id_buf);

        Ok(Self {
            user_key_pair,
            messages,
            max_dead_drop_id,
        })
    }

    pub fn write<W>(&self, writer: &mut W, key: &SecretBoxKey) -> anyhow::Result<()>
    where
        W: Write + Seek,
    {
        // Fill plaintext buffer
        let mut buf = Cursor::new(vec![]);

        buf.write_all(self.user_key_pair.public_key().as_bytes())?;

        // `as_bytes` has been added to main on x25519_dalek but it's not released yet
        buf.write_all(&self.user_key_pair.secret_key().to_bytes())?;

        self.messages.write(&mut buf)?;

        buf.write_all(self.max_dead_drop_id.to_be_bytes().as_ref())?;

        // Encrypt
        let ciphertext = SecretBox::encrypt(key, buf.into_inner())?;

        let before = writer.stream_position()?;
        writer.write_all(ciphertext.as_bytes())?;
        let after = writer.stream_position()?;
        assert_eq!((after - before) as usize, Self::SERIALIZED_LEN);

        Ok(())
    }
}
