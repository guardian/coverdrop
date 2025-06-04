use std::marker::PhantomData;

use crate::crypto::{anonymous_box, AnonymousBox};
use crate::Error;
use serde::{Deserialize, Serialize};
use serde_with::{
    base64::{Base64, Standard},
    formats::Unpadded,
    serde_as,
};
use sodiumoxide::crypto::secretbox::xsalsa20poly1305;

use super::keys::encryption::{traits, EncryptionKeyPair};
use super::keys::role::Role;
use super::Encryptable;

pub type SecretKey = xsalsa20poly1305::Key;

pub const KEY_LEN: usize = xsalsa20poly1305::KEYBYTES;
pub const NONCE_LEN: usize = xsalsa20poly1305::NONCEBYTES;
pub const CIPHERTEXT_OVERHEAD: usize = xsalsa20poly1305::MACBYTES;
pub const WRAPPED_KEY_LEN: usize = anonymous_box::PK_LEN + KEY_LEN + anonymous_box::TAG_LEN;

/// A variant of the [`AnonymousBox`] that allows multiple recipients such that any of the
/// recipients can decrypt the message. It internally generates a secret key that is used
/// to encrypt the payload using XSalsa20Poly1305. The secret key is then wrapped in multiple
/// independent [`sealed_box`] primitives for the respective recipients.
///
/// Like [`SecretBox`], `AnonymousBox` works with types that implement [`Encryptable`].
///
/// [`Encryptable`]: super::Encryptable
/// [`SecretBox`]: super::SecretBox
/// [`sealed_box`]: https://libsodium.gitbook.io/doc/public-key_cryptography/sealed_boxes
#[serde_as]
#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
#[serde(transparent, deny_unknown_fields)]
pub struct MultiAnonymousBox<T, const NUM_RECIPIENTS: usize> {
    #[serde_as(as = "Base64<Standard, Unpadded>")]
    wrapped_keys_and_ciphertext_and_tag: Vec<u8>,
    #[serde(skip)]
    marker: PhantomData<T>,
}

impl<T, const NUM_RECIPIENTS: usize> MultiAnonymousBox<T, NUM_RECIPIENTS> {
    /// Create a new `MultiAnonymousBox` from a byte vector without checking whether it's valid.
    pub fn from_vec_unchecked(bytes: Vec<u8>) -> MultiAnonymousBox<T, NUM_RECIPIENTS> {
        MultiAnonymousBox {
            wrapped_keys_and_ciphertext_and_tag: bytes,
            marker: PhantomData,
        }
    }

    pub fn as_bytes(&self) -> &Vec<u8> {
        &self.wrapped_keys_and_ciphertext_and_tag
    }

    pub fn encrypt<R>(
        recipients: [&impl traits::PublicEncryptionKey<R>; NUM_RECIPIENTS],
        data: T,
    ) -> Result<MultiAnonymousBox<T, NUM_RECIPIENTS>, Error>
    where
        R: Role,
        T: Encryptable,
    {
        let key = xsalsa20poly1305::gen_key();
        let bytes = data.as_unencrypted_bytes();

        let output_len = recipients.len() * WRAPPED_KEY_LEN + bytes.len() + CIPHERTEXT_OVERHEAD;
        let mut output: Vec<u8> = Vec::with_capacity(output_len);

        for recipient_pk in recipients {
            let wrapped_key = AnonymousBox::encrypt(recipient_pk, key.clone()).unwrap();
            output.extend_from_slice(wrapped_key.as_ref());
        }

        // since we always use fresh keys for each message, we can choose a constant nonce
        let nonce = xsalsa20poly1305::Nonce([0u8; NONCE_LEN]);
        let ciphertext = xsalsa20poly1305::seal(bytes, &nonce, &key);
        output.extend(ciphertext);

        assert_eq!(output.len(), output_len);
        Ok(MultiAnonymousBox {
            wrapped_keys_and_ciphertext_and_tag: output,
            marker: PhantomData,
        })
    }

    pub fn decrypt<PK, R>(
        decryption_key_pair: &EncryptionKeyPair<R, PK>,
        data: &MultiAnonymousBox<T, NUM_RECIPIENTS>,
    ) -> Result<T, Error>
    where
        PK: traits::PublicEncryptionKey<R>,
        T: Encryptable,
        R: Role,
    {
        let wrapped_keys =
            &data.wrapped_keys_and_ciphertext_and_tag[..NUM_RECIPIENTS * WRAPPED_KEY_LEN];
        let Some(matching_key) = Self::find_key(decryption_key_pair, wrapped_keys) else {
            return Err(Error::FailedToDecrypt);
        };

        let ciphertext =
            &data.wrapped_keys_and_ciphertext_and_tag[NUM_RECIPIENTS * WRAPPED_KEY_LEN..];

        // since we always use fresh keys for each message, we can choose a constant nonce
        let nonce = xsalsa20poly1305::Nonce([0u8; NONCE_LEN]);
        let Ok(plaintext_bytes) = xsalsa20poly1305::open(ciphertext, &nonce, &matching_key) else {
            return Err(Error::FailedToDecrypt);
        };
        T::from_unencrypted_bytes(plaintext_bytes)
    }

    fn find_key<PK, R>(
        decryption_key_pair: &EncryptionKeyPair<R, PK>,
        wrapped_keys: &[u8],
    ) -> Option<SecretKey>
    where
        PK: traits::PublicEncryptionKey<R>,
        R: Role,
    {
        for key_index in 0..NUM_RECIPIENTS {
            let offset = key_index * WRAPPED_KEY_LEN;
            let candidate = &wrapped_keys[offset..offset + WRAPPED_KEY_LEN];
            let anonymous_box = AnonymousBox::<SecretKey>::from_vec_unchecked(candidate.to_vec());
            if let Ok(key) = AnonymousBox::decrypt(decryption_key_pair, &anonymous_box) {
                return Some(key);
            }
        }
        None
    }

    // Ignoring the clippy `len_without_is_empty` since this isn't a container
    // it's a box, possibly `len` should be renamed.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.wrapped_keys_and_ciphertext_and_tag.len()
    }
}

impl<T, const NUM_RECIPIENTS: usize> AsRef<[u8]> for MultiAnonymousBox<T, NUM_RECIPIENTS> {
    fn as_ref(&self) -> &[u8] {
        &self.wrapped_keys_and_ciphertext_and_tag
    }
}

impl<T, const NUM_RECIPIENTS: usize> From<MultiAnonymousBox<T, NUM_RECIPIENTS>> for Vec<u8> {
    fn from(value: MultiAnonymousBox<T, NUM_RECIPIENTS>) -> Self {
        value.wrapped_keys_and_ciphertext_and_tag
    }
}

impl Encryptable for SecretKey {
    fn as_unencrypted_bytes(&self) -> &[u8] {
        self.0.as_slice()
    }

    fn from_unencrypted_bytes(bytes: Vec<u8>) -> Result<Self, Error> {
        SecretKey::from_slice(bytes.as_slice()).ok_or(Error::InvalidKey)
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        crypto::{
            keys::{encryption::UnsignedEncryptionKeyPair, role::Test},
            MultiAnonymousBox,
        },
        Error,
    };

    #[test]
    fn single_recipient_round_trip() -> Result<(), Error> {
        let input = "안녕하세요".to_owned();
        let recipient_key_pair = UnsignedEncryptionKeyPair::<Test>::generate();

        let recipients = [recipient_key_pair.public_key()];
        let encrypted: MultiAnonymousBox<String, 1> =
            MultiAnonymousBox::encrypt(recipients, input.clone())?;
        let decrypted: String = MultiAnonymousBox::decrypt(&recipient_key_pair, &encrypted)?;

        assert_eq!(input, decrypted);
        Ok(())
    }

    #[test]
    fn single_recipient_works_via_unchecked() -> Result<(), Error> {
        let input = "안녕하세요".to_owned();
        let recipient_key_pair = UnsignedEncryptionKeyPair::<Test>::generate();

        let recipients = [recipient_key_pair.public_key()];
        let encrypted: MultiAnonymousBox<String, 1> =
            MultiAnonymousBox::encrypt(recipients, input.clone())?;
        let raw_bytes = encrypted.wrapped_keys_and_ciphertext_and_tag;

        let from_bytes = MultiAnonymousBox::<String, 1>::from_vec_unchecked(raw_bytes);
        let decrypted = MultiAnonymousBox::decrypt(&recipient_key_pair, &from_bytes)?;

        assert_eq!(input, decrypted);
        Ok(())
    }

    #[test]
    fn single_recipient_fails_when_using_different_key() -> Result<(), Error> {
        let input = "안녕하세요".to_owned();

        let intended_recipient_key_pair = UnsignedEncryptionKeyPair::<Test>::generate();
        let other_recipient_key_pair = UnsignedEncryptionKeyPair::<Test>::generate();

        let recipients = [intended_recipient_key_pair.public_key()];
        let encrypted: MultiAnonymousBox<String, 1> =
            MultiAnonymousBox::encrypt(recipients, input)?;

        let decrypted: Result<String, Error> =
            MultiAnonymousBox::decrypt(&other_recipient_key_pair, &encrypted);

        assert!(
            matches!(decrypted, Err(Error::FailedToDecrypt)),
            "Making sure decryption failed, actual: {decrypted:?}"
        );
        Ok(())
    }

    #[test]
    fn multi_recipients_round_trip_works_for_all_of_them() -> Result<(), Error> {
        let input = "안녕하세요".to_owned();
        const NUM_RECIPIENTS: usize = 3;

        let recipients_key_pairs: Vec<_> = (0..NUM_RECIPIENTS)
            .map(|_| UnsignedEncryptionKeyPair::<Test>::generate())
            .collect();
        let recipients_pks: Vec<&_> = recipients_key_pairs
            .iter()
            .map(|x| x.public_key())
            .collect();

        let encrypted = MultiAnonymousBox::<String, NUM_RECIPIENTS>::encrypt(
            recipients_pks.as_slice().try_into().unwrap(),
            input.clone(),
        )?;

        for recipient_key_pair in recipients_key_pairs {
            let decrypted = MultiAnonymousBox::decrypt(&recipient_key_pair, &encrypted)?;

            assert_eq!(input, decrypted);
        }

        Ok(())
    }

    #[test]
    fn multi_recipients_round_trip_with_serialization() -> anyhow::Result<()> {
        let input = "안녕하세요".to_owned();
        const NUM_RECIPIENTS: usize = 3;

        let recipients_key_pairs: Vec<_> = (0..NUM_RECIPIENTS)
            .map(|_| UnsignedEncryptionKeyPair::<Test>::generate())
            .collect();
        let recipients_pks: Vec<&_> = recipients_key_pairs
            .iter()
            .map(|x| x.public_key())
            .collect();

        let encrypted = MultiAnonymousBox::<String, NUM_RECIPIENTS>::encrypt(
            recipients_pks.as_slice().try_into().unwrap(),
            input.clone(),
        )?;

        let serialized = serde_json::to_string(&encrypted).unwrap();
        let _deserialized =
            serde_json::from_str::<MultiAnonymousBox<String, NUM_RECIPIENTS>>(&serialized).unwrap();

        for recipient_key_pair in recipients_key_pairs {
            let decrypted = MultiAnonymousBox::decrypt(&recipient_key_pair, &encrypted)?;

            assert_eq!(input, decrypted);
        }

        Ok(())
    }
}
