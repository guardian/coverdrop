mod plain_mailbox_data;
mod secret_mailbox_data;

use std::{
    fs::OpenOptions,
    io::Read,
    path::{Path, PathBuf},
};

use crate::{
    api::models::journalist_id::JournalistIdentity,
    crypto::{
        keys::encryption::UnsignedEncryptionKeyPair,
        pbkdf::{derive_secret_box_key_with_configuration, generate_salt, Argon2Configuration},
        SecretBoxKey,
    },
    protocol::keys::UserKeyPair,
    time, FixedBuffer, FixedSizeMessageText,
};
use chrono::{DateTime, Utc};
use tracing::warn;

use self::{
    plain_mailbox_data::PlainMailboxData,
    secret_mailbox_data::{FixedMessageBuffer, SecretMailboxData},
};

use super::mailbox_message::MailboxMessage;

pub const MAX_MAILBOX_MESSAGES: usize = 128;

/// The mailbox stores private data for a users CoverDrop session
#[derive(Clone)]
pub struct UserMailbox {
    /// The path where the mailbox is stored
    path: PathBuf,
    // Mailbox key is not serialized, it is used when writing to disk
    key: SecretBoxKey,

    // Both plain and secret data are serialized to disk
    pub plain: PlainMailboxData,
    pub secret: SecretMailboxData,
}

impl UserMailbox {
    /// Create a new mailbox with a key derived from a password.
    ///
    /// The mailbox does not store any trust anchors - callers must supply the current
    /// organization public keys (e.g. from `trust_anchors::get_trust_anchors`) whenever they
    /// need to verify something, rather than relying on anchors pinned at mailbox creation time.
    pub fn new(password: &str, path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let user_key_pair = UnsignedEncryptionKeyPair::generate();

        Self::new_with_keys(password, user_key_pair, path)
    }

    pub fn new_with_keys(
        password: &str,
        user_key_pair: UserKeyPair,
        path: impl AsRef<Path>,
    ) -> anyhow::Result<Self> {
        let salt = generate_salt();
        let key =
            derive_secret_box_key_with_configuration(password, &salt, Argon2Configuration::V1)?;

        Ok(Self {
            path: path.as_ref().to_path_buf(),
            key,
            secret: SecretMailboxData {
                user_key_pair,
                messages: FixedBuffer::default(),
                max_dead_drop_created_at: DateTime::<Utc>::UNIX_EPOCH,
            },
            plain: PlainMailboxData { salt },
        })
    }

    pub fn load(path: impl AsRef<Path>, password: &str) -> anyhow::Result<Self> {
        let mut file = OpenOptions::new().read(true).open(&path)?;

        let plain = PlainMailboxData::read(&mut file)?;

        let mut encrypted_data = vec![];
        file.read_to_end(&mut encrypted_data)?;

        // first try to decrypt using the legacy Argon2 configuration
        let key_v0 = derive_secret_box_key_with_configuration(
            password,
            &plain.salt,
            Argon2Configuration::V0,
        )?;
        if let Ok(secret) = SecretMailboxData::deserialize(encrypted_data.clone(), &key_v0) {
            warn!("Loaded mailbox with legacy Argon2 configuration, consider re-encrypting with new configuration");
            return Ok(UserMailbox {
                path: path.as_ref().to_path_buf(),
                key: key_v0,
                secret,
                plain,
            });
        }

        // alternatively we try decrypting using the recent Argon2 configuration
        let key_v1 = derive_secret_box_key_with_configuration(
            password,
            &plain.salt,
            Argon2Configuration::V1,
        )?;
        let secret = SecretMailboxData::deserialize(encrypted_data, &key_v1)?;

        Ok(UserMailbox {
            path: path.as_ref().to_path_buf(),
            key: key_v1,
            secret,
            plain,
        })
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .open(&self.path)?;

        self.plain.write(&mut file)?;
        self.secret.write(&mut file, &self.key)?;

        Ok(())
    }

    pub fn user_key_pair(&self) -> &UserKeyPair {
        &self.secret.user_key_pair
    }

    pub fn messages(&self) -> &FixedMessageBuffer {
        &self.secret.messages
    }

    pub fn add_message_to_journalist_from_user(
        &mut self,
        to: &JournalistIdentity,
        message: &FixedSizeMessageText,
    ) {
        let now = time::now();
        let message = MailboxMessage::from_user_to_journalist(
            0,
            to,
            self.secret.user_key_pair.public_key(),
            message,
            now,
            false,
            None,
        );

        self.secret.messages.push(message);
    }

    pub fn add_message_to_user_from_journalist(
        &mut self,
        from: &JournalistIdentity,
        message: &FixedSizeMessageText,
    ) {
        let now = time::now();
        let message = MailboxMessage::from_journalist_to_user(
            0,
            self.secret.user_key_pair.public_key(),
            from,
            message,
            now,
            false,
            false,
            None,
        );

        self.secret.messages.push(message);
    }

    pub fn max_dead_drop_created_at(&self) -> DateTime<Utc> {
        self.secret.max_dead_drop_created_at
    }

    pub fn set_max_dead_drop_created_at(&mut self, created_at: DateTime<Utc>) {
        self.secret.max_dead_drop_created_at = created_at;
    }
}

impl Drop for UserMailbox {
    fn drop(&mut self) {
        if let Err(e) = self.save() {
            tracing::error!("Failed to save user mailbox during drop: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use crate::{api::models::journalist_id::JournalistIdentity, FixedSizeMessageText};

    use super::{
        plain_mailbox_data::PlainMailboxData, secret_mailbox_data::SecretMailboxData, UserMailbox,
    };

    const SERIALIZED_MAILBOX_SIZE: u64 =
        PlainMailboxData::SERIALIZED_LEN as u64 + SecretMailboxData::SERIALIZED_LEN as u64;

    #[test]
    fn can_roundtrip() -> anyhow::Result<()> {
        let test_files_dir = tempdir()?;
        let test_file_path = test_files_dir.path().join("test-mailbox");

        let mailbox_1 = UserMailbox::new("password", &test_file_path)?;

        mailbox_1.save()?;

        let mailbox_2 = UserMailbox::load(&test_file_path, "password")?;

        assert_eq!(
            mailbox_1.secret.user_key_pair.public_key(),
            mailbox_2.secret.user_key_pair.public_key()
        );

        // x25519_dalek doesn't implement Eq for secret keys, so match the bytes manually
        assert_eq!(
            mailbox_1.secret.user_key_pair.secret_key().to_bytes(),
            mailbox_2.secret.user_key_pair.secret_key().to_bytes()
        );
        assert_eq!(mailbox_1.secret.messages, mailbox_2.secret.messages);
        assert_eq!(
            mailbox_1.max_dead_drop_created_at(),
            chrono::DateTime::<chrono::Utc>::UNIX_EPOCH
        );
        assert_eq!(
            mailbox_1.max_dead_drop_created_at(),
            mailbox_2.max_dead_drop_created_at()
        );

        assert_eq!(mailbox_1.plain.salt, mailbox_2.plain.salt);

        Ok(())
    }

    #[test]
    fn always_the_same_size() -> anyhow::Result<()> {
        let test_files_dir = tempdir()?;
        let test_file_path = test_files_dir.path().join("test-mailbox");

        // Create initial
        let mailbox = UserMailbox::new("password", &test_file_path)?;

        mailbox.save()?;

        let expected_file_size = SERIALIZED_MAILBOX_SIZE;
        let actual_file_size = test_file_path.metadata()?.len();
        assert_eq!(
            actual_file_size,
            expected_file_size,
            "Size not expected! Expected {}, actual {}, difference {}",
            expected_file_size,
            actual_file_size,
            if expected_file_size > actual_file_size {
                format!(
                    "{} bytes under expected",
                    expected_file_size - actual_file_size
                )
            } else {
                format!(
                    "{} bytes over expected",
                    actual_file_size - expected_file_size
                )
            },
        );

        // Reload and insert a message
        let mut mailbox = UserMailbox::load(&test_file_path, "password")?;

        let to = JournalistIdentity::new("journalist")?;

        let message = FixedSizeMessageText::new(
            r#"
            The pen might not be mightier than the sword, but maybe the printing press was heavier than the siege weapon.

            Just a few words can change everything."#,
        )?;

        mailbox.add_message_to_journalist_from_user(&to, &message);

        mailbox.clone().save()?;
        assert_eq!(actual_file_size, SERIALIZED_MAILBOX_SIZE);

        Ok(())
    }

    #[test]
    fn overflowing_fixed_buffer_results_in_wrap_around() -> anyhow::Result<()> {
        let test_files_dir = tempdir()?;
        let test_file_path = test_files_dir.path().join("test-mailbox");

        // Create initial
        let mailbox = UserMailbox::new("password", &test_file_path)?;
        mailbox.save()?;

        let file_size = test_file_path.metadata()?.len();
        assert_eq!(file_size, SERIALIZED_MAILBOX_SIZE);

        // Insert messages
        let mut mailbox = UserMailbox::load(&test_file_path, "password")?;

        assert_eq!(mailbox.secret.messages.current_index(), 0);

        let to = JournalistIdentity::new("journalist")?;
        for i in 0..138 {
            let message = FixedSizeMessageText::new(&i.to_string())?;
            mailbox.add_message_to_journalist_from_user(&to, &message);
        }

        mailbox.save()?;
        let file_size = test_file_path.metadata()?.len();
        assert_eq!(file_size, SERIALIZED_MAILBOX_SIZE);

        // Assert we've only got messages 10..138
        let mailbox = UserMailbox::load(&test_file_path, "password")?;

        assert_eq!(mailbox.secret.messages.current_index(), 138);

        let mut msg_iter = mailbox.secret.messages.iter();

        // Strictly speaking we don't specify that the order is preserved during the wrap around.
        // But this is convenient for checking all the values we expect to be there are there.
        for i in 128..138 {
            let m = msg_iter.next().unwrap();
            assert_eq!(m.message.to_string()?, i.to_string());
        }
        for i in 10..128 {
            let m = msg_iter.next().unwrap();
            assert_eq!(m.message.to_string()?, i.to_string());
        }

        Ok(())
    }
}
