use std::hash::Hash;

use ed25519_dalek::{ed25519::SignatureBytes, Signature as Ed25519Signature, SIGNATURE_LENGTH};
use serde::{Deserialize, Serialize};
use sqlx::{database::HasValueRef, error::BoxDynError, Database, Decode};
use std::{hash::Hasher, marker::PhantomData};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Signature<T> {
    pub(crate) signature: Ed25519Signature,
    #[serde(skip)]
    pub(crate) marker: PhantomData<T>,
}

impl<T> Signature<T> {
    pub fn to_bytes(&self) -> SignatureBytes {
        self.signature.to_bytes()
    }

    pub fn from_vec_unchecked(vec: Vec<u8>) -> Signature<T> {
        let bytes: [u8; SIGNATURE_LENGTH] = vec[..].try_into().unwrap();

        Signature {
            signature: Ed25519Signature::from_bytes(&bytes),
            marker: PhantomData,
        }
    }
}

impl<'r, DB, T> Decode<'r, DB> for Signature<T>
where
    &'r [u8]: Decode<'r, DB>,
    DB: Database,
{
    fn decode(value: <DB as HasValueRef<'r>>::ValueRef) -> Result<Self, BoxDynError> {
        let value = <&[u8] as Decode<DB>>::decode(value)?;

        Ok(Signature::<T>::from_vec_unchecked(value.to_vec()))
    }
}

// Blanket implementations for signatures over different marker types.
// Without this all `Signables` must also implement `PartialEq`
impl<T> PartialEq for Signature<T> {
    fn eq(&self, other: &Self) -> bool {
        self.signature == other.signature
    }
}

impl<T> Eq for Signature<T> {}

impl<T> Hash for Signature<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.signature.to_bytes().hash(state);
    }
}
