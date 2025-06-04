use std::marker::PhantomData;

use crate::Error;
use serde::{Deserialize, Serialize};
use serde_with::{
    base64::{Base64, Standard},
    formats::Unpadded,
    serde_as,
};
use sodiumoxide::{
    crypto::box_::{self, PublicKey, SecretKey},
    utils::memzero,
};

use super::{
    keys::{
        encryption::{traits, SecretEncryptionKey},
        role::Role,
    },
    sodiumoxide_patches, Encryptable,
};

/// Intended for public key cryptography where both parties are known and can message each other.
///
/// Internally uses `libsodium`'s [`crypto_box`] primitives. Unlike `libsodium` we handle generating
/// a nonce and appending it to the outputted ciphertext and tag bytes.
///
/// The box does not handle the sharing public key. The public keys must be shared in some other way,
/// either passed in plaintext along with the ciphertex message or through public key infrastrcture.
///
/// Like [`SecretBox`], `AnonymousBox` works with types that implement [`Encryptable`].
///
/// [`Encryptable`]: super::Encryptable
/// [`SecretBox`]: super::SecretBox
/// [`crypto_box`]: https://libsodium.gitbook.io/doc/public-key_cryptography/authenticated_encryption
#[serde_as]
#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
#[serde(transparent, deny_unknown_fields)]
pub struct TwoPartyBox<T> {
    #[serde_as(as = "Base64<Standard, Unpadded>")]
    tag_ciphertext_and_nonce: Vec<u8>,
    #[serde(skip)]
    marker: PhantomData<T>,
}

impl<T> TwoPartyBox<T> {
    pub fn from_vec_unchecked(bytes: Vec<u8>) -> TwoPartyBox<T> {
        TwoPartyBox {
            tag_ciphertext_and_nonce: bytes,
            marker: PhantomData,
        }
    }

    pub fn as_bytes(&self) -> &Vec<u8> {
        &self.tag_ciphertext_and_nonce
    }

    pub fn encrypt<RecipientRole, SenderRole>(
        recipient_pk: &impl traits::PublicEncryptionKey<RecipientRole>,
        sender_sk: &SecretEncryptionKey<SenderRole>,
        data: T,
    ) -> Result<TwoPartyBox<T>, Error>
    where
        T: Encryptable,
        RecipientRole: Role,
        SenderRole: Role,
    {
        let nonce = box_::gen_nonce();

        let recipient_pk = PublicKey::from_slice(recipient_pk.as_bytes()).unwrap();
        let mut our_sk = SecretKey::from_slice(&sender_sk.to_bytes()).unwrap();

        let mut tag_and_ciphertext = sodiumoxide_patches::crypto_box::seal(
            data.as_unencrypted_bytes(),
            &nonce,
            &recipient_pk,
            &our_sk,
        )
        .map_err(|_| Error::FailedToEncrypt)?;

        tag_and_ciphertext.extend_from_slice(&nonce.0);

        let tag_ciphertext_and_nonce = tag_and_ciphertext;

        // Remove redundant copy of our secret key
        memzero(&mut our_sk.0);

        Ok(TwoPartyBox {
            tag_ciphertext_and_nonce,
            marker: PhantomData,
        })
    }

    pub fn decrypt<PK, SenderRole, RecipientRole>(
        sender_pk: &PK,
        recipient_sk: &SecretEncryptionKey<RecipientRole>,
        data: &TwoPartyBox<T>,
    ) -> Result<T, Error>
    where
        PK: traits::PublicEncryptionKey<SenderRole> + ?Sized,
        SenderRole: Role,
        RecipientRole: Role,
        T: Encryptable,
    {
        let sender_pk = PublicKey::from_slice(sender_pk.raw_public_key().as_bytes()).unwrap();
        let mut recipient_sk = SecretKey::from_slice(&recipient_sk.key.to_bytes()).unwrap();

        let bytes = &data.tag_ciphertext_and_nonce;
        let nonce_start = bytes.len() - box_::NONCEBYTES;
        let nonce = box_::Nonce::from_slice(&bytes[nonce_start..]).unwrap();

        let plaintext_bytes = box_::open(&bytes[..nonce_start], &nonce, &sender_pk, &recipient_sk)
            .map_err(|_| Error::FailedToDecrypt)?;

        memzero(&mut recipient_sk.0);

        let plaintext = T::from_unencrypted_bytes(plaintext_bytes)?;

        Ok(plaintext)
    }

    // Ignoring the clippy `len_without_is_empty` since this isn't a container
    // it's a box, possibly `len` should be renamed.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.tag_ciphertext_and_nonce.len()
    }
}

impl<T> AsRef<[u8]> for TwoPartyBox<T> {
    fn as_ref(&self) -> &[u8] {
        &self.tag_ciphertext_and_nonce
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        crypto::{
            keys::{encryption::UnsignedEncryptionKeyPair, role::Test},
            TwoPartyBox,
        },
        Error,
    };

    #[test]
    fn round_trip() -> Result<(), Error> {
        let input = "こんにちは".to_owned();
        let my_key_pair = UnsignedEncryptionKeyPair::<Test>::generate();
        let recipient_key_pair = UnsignedEncryptionKeyPair::<Test>::generate();

        let encrypted: TwoPartyBox<String> = TwoPartyBox::encrypt(
            recipient_key_pair.public_key(),
            my_key_pair.secret_key(),
            input.clone(),
        )?;
        let decrypted: String = TwoPartyBox::decrypt(
            recipient_key_pair.public_key(),
            my_key_pair.secret_key(),
            &encrypted,
        )?;

        assert_eq!(input, decrypted);
        Ok(())
    }

    #[test]
    fn works_via_unchecked() -> Result<(), Error> {
        let input = "こんにちは".to_owned();
        let my_key_pair = UnsignedEncryptionKeyPair::<Test>::generate();
        let recipient_key_pair = UnsignedEncryptionKeyPair::<Test>::generate();

        let encrypted: TwoPartyBox<String> = TwoPartyBox::encrypt(
            recipient_key_pair.public_key(),
            my_key_pair.secret_key(),
            input.clone(),
        )?;

        let raw_bytes = encrypted.tag_ciphertext_and_nonce;

        let from_bytes = TwoPartyBox::<String>::from_vec_unchecked(raw_bytes);
        let decrypted: String = TwoPartyBox::decrypt(
            recipient_key_pair.public_key(),
            my_key_pair.secret_key(),
            &from_bytes,
        )?;

        assert_eq!(input, decrypted);
        Ok(())
    }

    #[test]
    fn fails_when_using_different_recipient_key() -> Result<(), Error> {
        let input = "こんにちは".to_owned();

        let my_key_pair = UnsignedEncryptionKeyPair::<Test>::generate();

        let intended_recipient_key_pair = UnsignedEncryptionKeyPair::<Test>::generate();
        let other_recipient_key_pair = UnsignedEncryptionKeyPair::<Test>::generate();

        let encrypted: TwoPartyBox<String> = TwoPartyBox::encrypt(
            intended_recipient_key_pair.public_key(),
            my_key_pair.secret_key(),
            input,
        )?;

        let decrypted = TwoPartyBox::decrypt(
            other_recipient_key_pair.public_key(),
            my_key_pair.secret_key(),
            &encrypted,
        );

        assert!(
            matches!(decrypted, Err(Error::FailedToDecrypt)),
            "Making sure decryption failed, actual: {decrypted:?}"
        );

        Ok(())
    }

    #[test]
    fn fails_when_using_different_sender_key() -> Result<(), Error> {
        let input = "こんにちは".to_owned();

        let my_key_pair = UnsignedEncryptionKeyPair::<Test>::generate();
        let other_key_pair = UnsignedEncryptionKeyPair::<Test>::generate();
        let recipient_key_pair = UnsignedEncryptionKeyPair::<Test>::generate();

        let encrypted: TwoPartyBox<String> = TwoPartyBox::encrypt(
            recipient_key_pair.public_key(),
            my_key_pair.secret_key(),
            input,
        )?;

        let decrypted = TwoPartyBox::decrypt(
            recipient_key_pair.public_key(),
            other_key_pair.secret_key(),
            &encrypted,
        );

        assert!(
            matches!(decrypted, Err(Error::FailedToDecrypt)),
            "Making sure decryption failed, actual: {decrypted:?}"
        );

        Ok(())
    }
}
