use openmls_rust_crypto::RustCrypto;
use openmls_sqlx_storage::{Codec, SqliteStorageProvider};
use openmls_traits::OpenMlsProvider;
use serde::Serialize;
use sqlx::sqlite::SqliteConnection;

/// CBOR codec for serializing/deserializing OpenMLS data in SQLite storage.
#[derive(Default)]
pub(crate) struct CborCodec;

impl Codec for CborCodec {
    type Error = serde_cbor::Error;

    fn to_vec<T: Serialize + ?Sized>(value: &T) -> Result<Vec<u8>, Self::Error> {
        serde_cbor::to_vec(&value)
    }

    fn from_slice<T: serde::de::DeserializeOwned>(slice: &[u8]) -> Result<T, Self::Error> {
        serde_cbor::from_slice(slice)
    }
}

/// OpenMLS provider that uses RustCrypto for crypto operations and SQLite for persistent storage.
pub(crate) struct VaultProvider<'a> {
    // Using RustCrypto means that OpenMLS uses the same key exchange, AEAD, and signing algorithms
    // as the CoverDrop protocol.
    crypto: RustCrypto,
    storage: SqliteStorageProvider<'a, CborCodec>,
}

impl<'a> VaultProvider<'a> {
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        let storage = SqliteStorageProvider::<CborCodec>::new(conn);
        Self {
            crypto: RustCrypto::default(),
            storage,
        }
    }
}

impl<'a> OpenMlsProvider for VaultProvider<'a> {
    type CryptoProvider = RustCrypto;
    type RandProvider = RustCrypto;
    type StorageProvider = SqliteStorageProvider<'a, CborCodec>;

    fn crypto(&self) -> &Self::CryptoProvider {
        &self.crypto
    }

    fn rand(&self) -> &Self::RandProvider {
        &self.crypto
    }

    fn storage(&self) -> &Self::StorageProvider {
        &self.storage
    }
}
