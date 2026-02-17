use std::borrow::Cow;
use std::fmt::Display;
use std::marker::PhantomData;
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

use ed25519_dalek::SigningKey;
use hex_buffer_serde::Hex;
use regex::Regex;
use serde::de::DeserializeOwned;
use serde::Serialize;
use x25519_dalek::PublicKey as X25519PublicKey;
use x25519_dalek::StaticSecret as X25519SecretKey;

use crate::crypto::keys::role::Role;
use crate::crypto::Signature;

use super::public_key::PublicKey;
use super::{Ed25519PublicKey, Ed25519Signature};

pub(crate) struct PublicSigningKeyHex;

impl Hex<Ed25519PublicKey> for PublicSigningKeyHex {
    type Error = &'static str;

    fn create_bytes(value: &Ed25519PublicKey) -> Cow<'_, [u8]> {
        Cow::from(&value.as_bytes()[..])
    }

    fn from_bytes(bytes: &[u8]) -> Result<Ed25519PublicKey, Self::Error> {
        let bytes: [u8; 32] = bytes
            .try_into()
            .map_err(|_| "Provided public key byte array was the wrong length")?;

        Ed25519PublicKey::from_bytes(&bytes).map_err(|_| {
            "Failed to convert bytes to edwards point, probably point decompression issue"
        })
    }
}

pub(crate) struct SigningKeyPairHex;

impl Hex<SigningKey> for SigningKeyPairHex {
    type Error = &'static str;

    fn create_bytes(value: &SigningKey) -> Cow<'_, [u8]> {
        Cow::Owned(Vec::from(value.to_bytes()))
    }

    fn from_bytes(bytes: &[u8]) -> Result<SigningKey, Self::Error> {
        let bytes: [u8; 32] = bytes
            .try_into()
            .map_err(|_| "Provided signing key byte array was the wrong length")?;

        Ok(SigningKey::from_bytes(&bytes))
    }
}

pub(crate) struct PublicEncryptionKeyHex;

impl Hex<X25519PublicKey> for PublicEncryptionKeyHex {
    type Error = &'static str;

    fn create_bytes(value: &X25519PublicKey) -> Cow<'_, [u8]> {
        Cow::from(value.as_bytes().to_vec())
    }

    fn from_bytes(bytes: &[u8]) -> Result<X25519PublicKey, Self::Error> {
        let bytes: [u8; 32] = bytes
            .try_into()
            .map_err(|_| "Provided public key byte array was the wrong length")?;

        Ok(X25519PublicKey::from(bytes))
    }
}

pub(crate) struct SecretEncryptionKeyHex;

impl Hex<X25519SecretKey> for SecretEncryptionKeyHex {
    type Error = &'static str;

    fn create_bytes(value: &X25519SecretKey) -> Cow<'_, [u8]> {
        Cow::from(value.to_bytes().to_vec())
    }

    fn from_bytes(bytes: &[u8]) -> Result<X25519SecretKey, Self::Error> {
        let bytes: [u8; 32] = bytes
            .try_into()
            .map_err(|_| "Provided secret key byte array was the wrong length")?;

        Ok(X25519SecretKey::from(bytes))
    }
}

pub(crate) struct SignatureHex;

impl<T> Hex<Signature<T>> for SignatureHex {
    type Error = &'static str;

    fn create_bytes(value: &Signature<T>) -> Cow<'_, [u8]> {
        Cow::from(value.signature.to_vec())
    }

    fn from_bytes(bytes: &[u8]) -> Result<Signature<T>, Self::Error> {
        let bytes: &[u8; 64] = bytes
            .try_into()
            .map_err(|_| "Could not convert byte slice into fixed length byte array")?;

        let signature = Ed25519Signature::from_bytes(bytes);

        Ok(Signature {
            signature,
            marker: PhantomData,
        })
    }
}

pub enum StorableKeyMaterialType {
    PublicKey,
    KeyPair,
}

impl Display for StorableKeyMaterialType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PublicKey => write!(f, "public key"),
            Self::KeyPair => write!(f, "key pair"),
        }
    }
}

// This constant also used in `integration-tests/scripts/refresh_keys.sh`
const KEY_ID_HEX_LEN: usize = 8;

fn key_material_file_path(
    base_path: impl AsRef<Path>,
    entity: &str,
    public_key_hex: &str,
    key_type: StorableKeyMaterialType,
) -> PathBuf {
    let mut path = base_path.as_ref().to_owned();

    let key_type_modifier = match key_type {
        StorableKeyMaterialType::PublicKey => "pub",
        StorableKeyMaterialType::KeyPair => "keypair",
    };

    path.push(format!(
        "{}-{}.{}.json",
        entity,
        &public_key_hex[..KEY_ID_HEX_LEN],
        key_type_modifier
    ));
    path
}

fn key_material_path_regex(entity: &str, key_type: StorableKeyMaterialType) -> String {
    let key_type_modifier = match key_type {
        StorableKeyMaterialType::PublicKey => "pub",
        StorableKeyMaterialType::KeyPair => "keypair",
    };

    format!("^{entity}-([0-9a-fA-F]{{{KEY_ID_HEX_LEN}}}).{key_type_modifier}.json$")
}

// st_mode for the keys

// File type bits (0o10____)
// -------------------------
// Regular file

// File mode bits (0o__0___)
// -------------------------
// Sticky bit - off
// SUID       - off
// SGID       - off

// Permission bits (0o___600)
// --------------------------
// User       - read, write
// Group      - off
// All        - off
const KEYS_UNIX_ST_MODE: u32 = 0o100600;

#[cfg(unix)]
pub fn set_key_permissions(file_path: impl AsRef<Path>) {
    use std::{fs::Permissions, os::unix::fs::PermissionsExt};

    let permissions = Permissions::from_mode(KEYS_UNIX_ST_MODE);

    if let Err(e) = fs::set_permissions(&file_path, permissions) {
        tracing::warn!(
            "Failed to set file permissions for key '{}', {}",
            file_path.as_ref().display(),
            e
        );
    }
}

#[cfg(not(unix))]
pub fn set_key_permissions(file: &DirEntry) {
    tracing::warn!("Setting key permissions is not implemented for non-UNIX deployments");
}

/// Reusable code for storing either a public key or a key pair
pub trait StorableKeyMaterial<'a, KeyRole: Role>:
    Sized + Serialize + DeserializeOwned + PublicKey
{
    /// Associated constant which switches the extension when saving the stored key material
    const TYPE: StorableKeyMaterialType;

    fn file_name(&self) -> String {
        let file_path = key_material_file_path(
            PathBuf::new(),
            KeyRole::entity_name(),
            &self.public_key_hex(),
            Self::TYPE,
        );

        file_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned()
    }

    /// Save the key material to disk, returns the path generated
    fn save_to_disk(&self, path: impl AsRef<Path>) -> anyhow::Result<PathBuf> {
        let file_path = key_material_file_path(
            path.as_ref(),
            KeyRole::entity_name(),
            &self.public_key_hex(),
            Self::TYPE,
        );

        let json = serde_json::to_string(self)?;

        fs::write(&file_path, json)?;

        set_key_permissions(&file_path);

        Ok(file_path)
    }

    fn load_from_directory(path: impl AsRef<Path>) -> anyhow::Result<Vec<Self>> {
        if !path.as_ref().is_dir() {
            anyhow::bail!("Provided path was not a directory");
        }

        Self::load_from_disk_with_entity_name(path, KeyRole::entity_name())
    }

    /// Load a file skipping the permissions check.
    ///
    /// The only time this is currently used is when adding a trust anchor public key to the journalist client
    /// since we don't expect journalist users to have their new public key's permission locked down.
    ///
    /// Should never be used by a service.
    fn load_from_file_skip_permissions_check(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();

        let reader = File::open(path).map_err(|e| {
            tracing::error!("Failed to read key material from {}: {}", path.display(), e);
            e
        })?;

        let key = serde_json::from_reader::<_, Self>(reader).inspect_err(|e| {
            tracing::error!(
                "Failed to serialize key material JSON for {}: {:?}",
                path.display(),
                e
            );
        })?;

        Ok(key)
    }

    fn load_from_file(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();
        Self::permissions_check(path);

        Self::load_from_file_skip_permissions_check(path)
    }

    #[cfg(unix)]
    fn permissions_check(path: impl AsRef<Path>) {
        use std::os::unix::fs::PermissionsExt;

        let path = path.as_ref();

        let permissions_check = path.metadata().map(|metadata| {
            let permissions = metadata.permissions();
            permissions.mode()
        });

        match permissions_check {
            Ok(mode) => {
                if mode != KEYS_UNIX_ST_MODE {
                    let message = format!("Key {} does not have the expected file permissions (expected: 100600, actual: {:o})", path.display(), mode);

                    tracing::error!(message);
                    panic!("{}", message)
                }
            }
            Err(e) => {
                panic!(
                    "Failed to read file permissions for key '{}', {}",
                    path.display(),
                    e
                );
            }
        }
    }

    #[cfg(not(unix))]
    fn permissions_check(file: &DirEntry) {
        tracing::warn!("Key permissions checks are not implemented for non-UNIX deployments");
    }

    fn load_from_disk_with_entity_name(
        path: impl AsRef<Path>,
        entity_name: &str,
    ) -> anyhow::Result<Vec<Self>> {
        let filter_regex = key_material_path_regex(entity_name, Self::TYPE);
        let filter_regex = Regex::new(&filter_regex)?;

        tracing::debug!(
            "Reading {} {} from directory {}",
            entity_name,
            Self::TYPE,
            &path.as_ref().display()
        );

        let keys = fs::read_dir(path)?
            .flat_map(|file| {
                let file = file
                    .map_err(|e| {
                        tracing::error!("Error while listing file {}", e);
                        e
                    })
                    .ok()?;

                let file_name = file.file_name();
                let file_name = file_name.to_string_lossy();

                if filter_regex.is_match(file_name.as_ref()) {
                    Some(file)
                } else {
                    None
                }
            })
            .flat_map(|file| -> anyhow::Result<Self> { Self::load_from_file(file.path()) })
            .collect::<Vec<Self>>();

        Ok(keys)
    }
}

#[cfg(test)]
mod tests {
    use hex_buffer_serde::Hex;

    use crate::crypto::Signature;

    use super::{key_material_path_regex, SignatureHex, StorableKeyMaterialType};

    #[test]
    pub fn signature_hex_from_bytes() {
        let short = [0; 8];
        let parse: Result<Signature<Vec<u8>>, &'static str> = SignatureHex::from_bytes(&short[..]);

        assert!(
            parse.is_err(),
            "expected short byte array to cause an error"
        );

        let correct = [0; 64];
        let parse: Result<Signature<Vec<u8>>, &'static str> =
            SignatureHex::from_bytes(&correct[..]);

        assert!(
            parse.is_ok(),
            "expected correctly sized byte array to be parsed without error"
        );

        let long = [0; 128];
        let parse: Result<Signature<Vec<u8>>, &'static str> = SignatureHex::from_bytes(&long[..]);

        assert!(parse.is_err(), "expected long byte array to cause an error");
    }

    #[test]
    pub fn extra_extensions_dont_match() {
        // While working on ceremonies we use `age` for file encryption but our previous regex
        // would pick those up and we'd get a serde failure. This test prevents the fix to that
        // from regressing.
        let ok = "foo-01234567.keypair.json";
        let bad = "foo-01234567.keypair.json.age";

        let regex = key_material_path_regex("foo", StorableKeyMaterialType::KeyPair);

        let re = regex::Regex::new(&regex).unwrap();

        assert!(re.is_match(ok), "regex should match valid filename");
        assert!(
            !re.is_match(bad),
            "regex should not match filename with extra extension"
        );
    }
}
