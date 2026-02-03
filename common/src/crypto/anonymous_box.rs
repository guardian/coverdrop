use std::marker::PhantomData;

use serde::{Deserialize, Serialize};
use serde_with::{
    base64::{Base64, Standard},
    formats::Unpadded,
    serde_as,
};
use sodiumoxide::crypto::box_::{PublicKey, SecretKey};
use sodiumoxide::utils::memzero;
use sqlx::error::BoxDynError;
use sqlx::{Database, Decode};

use crate::protocol::constants;
use crate::Error;

use super::keys::encryption::{traits, EncryptionKeyPair};
use super::keys::role::Role;
use super::{sodiumoxide_patches, Encryptable};

pub const PK_LEN: usize = constants::ED25519_PUBLIC_KEY_LEN;
pub const TAG_LEN: usize = constants::POLY1305_AUTH_TAG_LEN;

/// Used for public key cryptography using ephemeral X25519 key exchange followed by XSalsa20Poly1305.
/// Internally uses `libsodium`'s `sealed_box` primitive, for interoperability with other platforms.
///
/// The byte array contains the ciphertext, AEAD tag and ephemeral public key.
/// Nonces are not stored since they are created by hashing the ephemeral and recipient public keys using BLAKE2.
///
/// Like [`SecretBox`], `AnonymousBox` works with types that implement [`Encryptable`].
///
/// [`Encryptable`]: super::Encryptable
/// [`SecretBox`]: super::SecretBox
/// [`sealed_box`]: https://libsodium.gitbook.io/doc/public-key_cryptography/sealed_boxes
#[serde_as]
#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
#[serde(transparent, deny_unknown_fields)]
pub struct AnonymousBox<T> {
    #[serde_as(as = "Base64<Standard, Unpadded>")]
    pk_tag_and_ciphertext: Vec<u8>,
    #[serde(skip)]
    marker: PhantomData<T>,
}

impl<T> AnonymousBox<T> {
    /// Create a new `AnonymousBox` from a byte vector without checking if it's valid.
    /// See [`SecretBox::from_vec_unchecked`] for more details.
    ///
    /// [`SecretBox::from_vec_unchecked`]: super::SecretBox::from_vec_unchecked
    pub fn from_vec_unchecked(bytes: Vec<u8>) -> AnonymousBox<T> {
        AnonymousBox {
            pk_tag_and_ciphertext: bytes,
            marker: PhantomData,
        }
    }

    pub fn as_bytes(&self) -> &Vec<u8> {
        &self.pk_tag_and_ciphertext
    }

    pub fn encrypt<R>(
        recipient_pk: &impl traits::PublicEncryptionKey<R>,
        data: T,
    ) -> Result<AnonymousBox<T>, Error>
    where
        R: Role,
        T: Encryptable,
    {
        let bytes = data.as_unencrypted_bytes();
        let pk = PublicKey::from_slice(recipient_pk.raw_public_key().as_bytes())
            .ok_or(Error::InvalidPublicKeyBytes)?;
        let pk_tag_and_ciphertext = sodiumoxide_patches::sealed_box::seal(bytes, &pk)
            .map_err(|_| Error::FailedToEncrypt)?;

        Ok(AnonymousBox {
            pk_tag_and_ciphertext,
            marker: PhantomData,
        })
    }

    pub fn decrypt<PK, R>(
        decryption_key_pair: &EncryptionKeyPair<R, PK>,
        data: &AnonymousBox<T>,
    ) -> Result<T, Error>
    where
        PK: traits::PublicEncryptionKey<R>,
        T: Encryptable,
        R: Role,
    {
        let ciphertext = &data.pk_tag_and_ciphertext;
        let pk =
            PublicKey::from_slice(decryption_key_pair.public_key().raw_public_key().as_bytes())
                .unwrap();
        let mut sk = SecretKey::from_slice(&decryption_key_pair.secret_key().to_bytes()).unwrap();

        let plaintext_bytes = sodiumoxide::crypto::sealedbox::open(ciphertext, &pk, &sk)
            .map_err(|_| Error::FailedToDecrypt)?;

        let plaintext = T::from_unencrypted_bytes(plaintext_bytes)?;

        memzero(&mut sk.0);

        Ok(plaintext)
    }

    // Ignoring the clippy `len_without_is_empty` since this isn't a container
    // it's a box, possibly `len` should be renamed.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.pk_tag_and_ciphertext.len()
    }
}

impl<T> AsRef<[u8]> for AnonymousBox<T> {
    fn as_ref(&self) -> &[u8] {
        &self.pk_tag_and_ciphertext
    }
}

impl<T> From<AnonymousBox<T>> for Vec<u8> {
    fn from(value: AnonymousBox<T>) -> Self {
        value.pk_tag_and_ciphertext
    }
}

impl<'r, DB, T> Decode<'r, DB> for AnonymousBox<T>
where
    &'r [u8]: Decode<'r, DB>,
    DB: Database,
{
    fn decode(value: DB::ValueRef<'r>) -> Result<Self, BoxDynError> {
        let value = <&[u8] as Decode<DB>>::decode(value)?;

        Ok(AnonymousBox::<T>::from_vec_unchecked(value.to_vec()))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        crypto::{
            keys::{encryption::UnsignedEncryptionKeyPair, role::Test},
            AnonymousBox,
        },
        Error,
    };

    #[test]
    fn round_trip() -> Result<(), Error> {
        let input = "안녕하세요".to_owned();
        let recipient_key_pair = UnsignedEncryptionKeyPair::<Test>::generate();

        let encrypted: AnonymousBox<String> =
            AnonymousBox::encrypt(recipient_key_pair.public_key(), input.clone())?;
        let decrypted: String = AnonymousBox::decrypt(&recipient_key_pair, &encrypted)?;

        assert_eq!(input, decrypted);
        Ok(())
    }

    #[test]
    fn works_via_unchecked() -> Result<(), Error> {
        let input = "안녕하세요".to_owned();
        let recipient_key_pair = UnsignedEncryptionKeyPair::<Test>::generate();

        let encrypted: AnonymousBox<String> =
            AnonymousBox::encrypt(recipient_key_pair.public_key(), input.clone())?;

        let raw_bytes = encrypted.pk_tag_and_ciphertext;

        let from_bytes = AnonymousBox::<String>::from_vec_unchecked(raw_bytes);
        let decrypted: String = AnonymousBox::decrypt(&recipient_key_pair, &from_bytes)?;

        assert_eq!(input, decrypted);
        Ok(())
    }

    #[test]
    fn fails_when_using_different_key() -> Result<(), Error> {
        let input = "안녕하세요".to_owned();

        let intended_recipient_key_pair = UnsignedEncryptionKeyPair::<Test>::generate();
        let other_recipient_key_pair = UnsignedEncryptionKeyPair::<Test>::generate();

        let encrypted: AnonymousBox<String> =
            AnonymousBox::encrypt(intended_recipient_key_pair.public_key(), input)?;

        let decrypted: Result<String, Error> =
            AnonymousBox::decrypt(&other_recipient_key_pair, &encrypted);

        assert!(
            matches!(decrypted, Err(Error::FailedToDecrypt)),
            "Making sure decryption failed, actual: {decrypted:?}"
        );

        Ok(())
    }

    #[test]
    fn round_trip_with_serialization() -> anyhow::Result<()> {
        let input = "안녕하세요".to_owned();

        let recipient_key_pair = UnsignedEncryptionKeyPair::<Test>::generate();

        let encrypted: AnonymousBox<String> =
            AnonymousBox::encrypt(recipient_key_pair.public_key(), input.clone())?;

        let serialized = serde_json::to_string(&encrypted).unwrap();

        let deserialized = serde_json::from_str::<AnonymousBox<String>>(&serialized).unwrap();

        let decrypted: String = AnonymousBox::decrypt(&recipient_key_pair, &deserialized)?;

        assert_eq!(decrypted, input);

        Ok(())
    }
}
