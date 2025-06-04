use crate::Error;

/// A structure that can be **directly** encrypted.
///
/// This is useful when you don't want to incur some kind of serializion costs, such as
/// increased message size or performance penalties.
///
/// Many structures will require some kind of serialization before they can be encrypted.
/// In these cases you can [`derive Serialize and Deserialize`] on your struct and
/// call the appropate [`Serialized`] functions.
///
/// [`derive Serialize and Deserialize`]: https://serde.rs/derive.html
/// [`SecretBox`]: crate::crypto::SecretBox
/// [`Serialized`]: crate::Serialized
pub trait Encryptable: Sized {
    fn as_unencrypted_bytes(&self) -> &[u8];
    fn from_unencrypted_bytes(bytes: Vec<u8>) -> Result<Self, Error>;
}

impl Encryptable for String {
    fn as_unencrypted_bytes(&self) -> &[u8] {
        self.as_bytes()
    }

    fn from_unencrypted_bytes(bytes: Vec<u8>) -> Result<Self, Error> {
        String::from_utf8(bytes).map_err(|e| Error::General(format!("{e}")))
    }
}

impl Encryptable for Vec<u8> {
    fn as_unencrypted_bytes(&self) -> &[u8] {
        self.as_slice()
    }

    fn from_unencrypted_bytes(bytes: Vec<u8>) -> Result<Self, Error> {
        Ok(bytes)
    }
}
