use std::marker::PhantomData;

use chacha20poly1305::{
    aead::{Aead, Key, OsRng},
    AeadCore, KeyInit, XChaCha20Poly1305, XNonce,
};

use crate::protocol::constants;
use crate::Error;

use super::encryptable::Encryptable;

pub type SecretBoxKey = Key<XChaCha20Poly1305>;

pub const KEY_LEN: usize = 32;
pub const NONCE_LEN: usize = 24;
pub const TAG_LEN: usize = constants::POLY1305_AUTH_TAG_LEN;

pub const SECRET_BOX_FOOTER_LEN: usize = NONCE_LEN + TAG_LEN;

/// Serialized values which are encrypted by a secret key. Uses XChaCha20Poly1305 internally.
/// Unlike NaCl-like libraries it also handles the nonce, by appending it to the end of the buffer.
///
/// Encryption and decryption and be performed on structures which implement the [`Encryptable`] trait.
///
/// [`Encryptable`]: super::Encryptable
pub struct SecretBox<T> {
    ciphertext_tag_and_nonce: Vec<u8>,
    marker: PhantomData<T>,
}

impl<T> SecretBox<T> {
    /// Create a new `SecretBox` from an input `Vec<u8>` without checking if it's valid
    /// ciphertext or that the type of the encrypted data is correct.
    ///
    /// This is used when you're constructing a `SecretBox` with data from outside of Rust code, such
    /// as from a network or filesystem.
    ///
    /// If the provided data is not real AE tagged ciphertext if will almost certainly fail to decrypt.
    ///
    /// If the provided data is real ciphertext, but the types are different the error behaviour is
    /// dependent on the serialization function.
    pub fn from_vec_unchecked(bytes: Vec<u8>) -> SecretBox<T> {
        SecretBox {
            ciphertext_tag_and_nonce: bytes,
            marker: PhantomData,
        }
    }

    pub fn encrypt(key: &SecretBoxKey, plaintext: T) -> Result<SecretBox<T>, Error>
    where
        T: Encryptable,
    {
        let bytes = plaintext.as_unencrypted_bytes();

        let ciphertext_tag_and_nonce = Self::encrypt_bytes(key, bytes)?;

        Ok(SecretBox {
            ciphertext_tag_and_nonce,
            marker: PhantomData,
        })
    }

    fn encrypt_bytes(key: &SecretBoxKey, bytes: &[u8]) -> Result<Vec<u8>, Error> {
        let aead = XChaCha20Poly1305::new(key);

        let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);

        let mut ciphertext_tag_and_nonce = aead.encrypt(&nonce, bytes)?;
        ciphertext_tag_and_nonce.extend_from_slice(&nonce);

        Ok(ciphertext_tag_and_nonce)
    }

    pub fn decrypt(key: &SecretBoxKey, data: SecretBox<T>) -> Result<T, Error>
    where
        T: Encryptable,
    {
        let plaintext_bytes = Self::decrypt_bytes(key, &data.ciphertext_tag_and_nonce)?;
        let plaintext = T::from_unencrypted_bytes(plaintext_bytes)?;

        Ok(plaintext)
    }

    fn decrypt_bytes(key: &SecretBoxKey, secretbox_bytes: &[u8]) -> Result<Vec<u8>, Error> {
        let aead = XChaCha20Poly1305::new(key);

        let nonce_start = secretbox_bytes.len() - NONCE_LEN;

        let nonce = XNonce::from_slice(&secretbox_bytes[nonce_start..]);
        let ciphertext = &secretbox_bytes[..nonce_start];

        let plaintext_bytes = aead.decrypt(nonce, ciphertext)?;

        Ok(plaintext_bytes)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.ciphertext_tag_and_nonce
    }
}

#[cfg(test)]
mod tests {
    use chacha20poly1305::Key;

    use crate::{crypto::secret_box::SecretBox, Error};

    #[test]
    fn works_with_types() {
        let input = "hello world".to_owned();
        let key = Key::from_slice(&[0; 32]);
        let encrypted: SecretBox<String> = SecretBox::encrypt(key, input.clone()).unwrap();
        let decrypted: String = SecretBox::decrypt(key, encrypted).unwrap();

        assert_eq!(input, decrypted);
    }

    #[test]
    fn works_via_unchecked() {
        // Setup to get some valid encrypted bytes
        let input = "hello world".to_owned();
        let key = Key::from_slice(&[0; 32]);
        let encrypted: SecretBox<String> = SecretBox::encrypt(key, input.clone()).unwrap();

        let raw_bytes: Vec<u8> = encrypted.ciphertext_tag_and_nonce;
        let from_bytes: SecretBox<String> = SecretBox::from_vec_unchecked(raw_bytes);
        let decrypted: String = SecretBox::decrypt(key, from_bytes).unwrap();

        assert_eq!(input, decrypted);
    }

    #[test]
    fn fails_when_using_different_key() -> Result<(), Error> {
        let input = "hello world".to_owned();
        let key = Key::from_slice(&[0; 32]);
        let encrypted: SecretBox<String> = SecretBox::encrypt(key, input)?;

        let different_key = Key::from_slice(&[1; 32]);
        let decrypted = SecretBox::decrypt(different_key, encrypted);

        assert!(
            matches!(decrypted, Err(Error::Aead(_))),
            "Making sure decryption failed, actual: {decrypted:?}"
        );

        Ok(())
    }
}
