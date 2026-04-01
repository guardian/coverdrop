use serde::{Deserialize, Serialize};
use sqlx::Type;
use std::ops::Deref;

/// Wrapper type for TLS-serialized MLS messages and Key Packages.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct TlsSerialized(Vec<u8>);

impl TlsSerialized {
    /// deserialize the inner bytes as a type implementing tls_codec::Deserialize.
    pub fn deserialize<T: openmls::prelude::tls_codec::Deserialize>(
        &self,
    ) -> Result<T, openmls::prelude::tls_codec::Error> {
        T::tls_deserialize_exact(self.0.as_slice())
    }

    /// serialize a type implementing tls_codec::Serialize into TlsSerialized.
    pub fn serialize<T: openmls::prelude::tls_codec::Serialize>(
        value: &T,
    ) -> Result<Self, openmls::prelude::tls_codec::Error> {
        let bytes = value.tls_serialize_detached()?;
        Ok(Self(bytes))
    }
}

impl Deref for TlsSerialized {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
