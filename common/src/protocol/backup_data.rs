use crate::api::models::journalist_id::JournalistIdentity;
use crate::crypto::keys::signing::traits::PublicSigningKey;
use crate::crypto::keys::signing::{SignedPublicSigningKey, SignedSigningKeyPair};
use crate::crypto::keys::untrusted::signing::UntrustedSignedPublicSigningKey;
use crate::crypto::{
    AnonymousBox, Encryptable, SecretBox, SecretSharingShare, Signable, Signature,
};
use crate::padded_byte_vector::SteppingPaddedByteVector;
use crate::protocol::roles::JournalistId;
use crate::Error;
use anyhow::Context;
use base64::prelude::BASE64_STANDARD_NO_PAD;
use base64::Engine;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Decode;
use std::fmt::Debug;

/// We pad the Sentinel backups to the next multiple of 1 MiB.
pub const BACKUP_PADDING_STEPS: usize = 1024 * 1024;

pub type BackupEncryptedPaddedVault = SecretBox<SteppingPaddedByteVector<BACKUP_PADDING_STEPS>>;

/// Helper for (de)serializing the `BackupEncryptedPaddedVault` as a byte array. We use this
/// instead of implementing (de)serialization directly on the `SecretBox` type, because we want to
/// use a more compact byte array representation instead of the default JSON representation that we
/// have adopted for similar types.
#[derive(Serialize, Deserialize)]
struct VaultBytes(Vec<u8>);

impl BackupEncryptedPaddedVault {
    pub fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let helper = VaultBytes(self.as_bytes().to_vec());
        helper.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let helper = VaultBytes::deserialize(deserializer)?;
        Ok(BackupEncryptedPaddedVault::from_vec_unchecked(helper.0))
    }
}

/// A secret share encrypted under another journalists messaging key.
pub type EncryptedSecretShare = AnonymousBox<SecretSharingShare>;

impl Encryptable for EncryptedSecretShare {
    fn as_unencrypted_bytes(&self) -> &[u8] {
        self.as_bytes()
    }

    fn from_unencrypted_bytes(bytes: Vec<u8>) -> Result<Self, Error> {
        Ok(AnonymousBox::<SecretSharingShare>::from_vec_unchecked(
            bytes,
        ))
    }
}

impl EncryptedSecretShare {
    pub fn from_base64_string(s: &str) -> anyhow::Result<Self> {
        let bytes = BASE64_STANDARD_NO_PAD.decode(s).with_context(|| {
            format!(
                "Failed to decode EncryptedSecretShare from base64 string: {}",
                s
            )
        })?;
        Ok(AnonymousBox::<SecretSharingShare>::from_vec_unchecked(
            bytes,
        ))
    }

    pub fn to_base64_string(&self) -> String {
        BASE64_STANDARD_NO_PAD.encode(self.as_bytes())
    }
}

/// An encrypted secret share additionally encrypted under the backup admin encryption key.
pub type BackupEncryptedSecretShare = AnonymousBox<EncryptedSecretShare>;

impl BackupEncryptedSecretShare {
    pub fn from_base64_string(s: &str) -> anyhow::Result<Self> {
        let bytes = BASE64_STANDARD_NO_PAD.decode(s).with_context(|| {
            format!(
                "Failed to decode BackupEncryptedSecretShare from base64 string: {}",
                s
            )
        })?;
        Ok(AnonymousBox::<EncryptedSecretShare>::from_vec_unchecked(
            bytes,
        ))
    }

    pub fn to_base64_string(&self) -> String {
        BASE64_STANDARD_NO_PAD.encode(self.as_bytes())
    }
}

/// The data structure that is backed up by a journalist and can be used to recover their vault.
#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Debug)]
pub struct BackupData {
    pub journalist_identity: JournalistIdentity,
    #[serde(with = "BackupEncryptedPaddedVault")]
    pub backup_encrypted_padded_vault: BackupEncryptedPaddedVault,
    pub wrapped_encrypted_shares: Vec<BackupEncryptedSecretShare>,
    pub created_at: DateTime<Utc>,
    pub recovery_contacts: Vec<JournalistIdentity>,
}

impl BackupData {
    pub(crate) fn to_bytes(&self) -> anyhow::Result<BackupDataBytes> {
        serde_cbor::to_vec(self)
            .context("Failed to serialize BackupData to bytes")
            .map(BackupDataBytes)
    }

    pub(crate) fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        serde_cbor::from_slice(bytes).context("Failed to deserialize BackupData from bytes")
    }

    pub fn to_backup_data_with_signature(
        &self,
        journalist_identity_key_pair: &SignedSigningKeyPair<JournalistId>,
    ) -> anyhow::Result<BackupDataWithSignature> {
        let bytes = self.to_bytes()?;
        let signature = journalist_identity_key_pair.sign(&bytes);
        Ok(BackupDataWithSignature {
            backup_data_bytes: bytes,
            backup_data_signature: signature,
            signed_with: journalist_identity_key_pair
                .public_key()
                .clone()
                .to_untrusted(),
        })
    }
}

/// Helper for (de)serializing the `BackupData` as a byte array that can be signed/verified.
#[derive(Serialize, Deserialize, Clone, Decode, PartialEq, Eq, Debug)]
pub struct BackupDataBytes(pub Vec<u8>);

impl BackupDataBytes {
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl Signable for BackupDataBytes {
    fn as_signable_bytes(&self) -> &[u8] {
        self.0.as_slice()
    }
}

/// A `BackupData` along with a signature by the journalist's identity key. This allows
/// verification that the backup was indeed created by the journalist who owns the identity.
/// The fields are non-public to ensure that callsites need to verify.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct BackupDataWithSignature {
    backup_data_bytes: BackupDataBytes,
    backup_data_signature: Signature<BackupDataBytes>,
    signed_with: UntrustedSignedPublicSigningKey<JournalistId>,
}

impl BackupDataWithSignature {
    pub fn new(
        backup_data_bytes: BackupDataBytes,
        backup_data_signature: Signature<BackupDataBytes>,
        journalist_identity_key_pair: UntrustedSignedPublicSigningKey<JournalistId>,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            backup_data_bytes,
            backup_data_signature,
            signed_with: journalist_identity_key_pair,
        })
    }

    pub fn from_vec_unchecked(data: Vec<u8>) -> anyhow::Result<Self> {
        let signed_backup_data: BackupDataWithSignature = serde_json::from_slice(&data)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize SignedBackupData: {}", e))?;
        Ok(signed_backup_data)
    }

    pub fn signed_with(&self) -> &UntrustedSignedPublicSigningKey<JournalistId> {
        &self.signed_with
    }

    pub fn to_verified(
        self,
        journalist_identity_public_key: &SignedPublicSigningKey<JournalistId>,
        now: DateTime<Utc>,
    ) -> anyhow::Result<VerifiedBackupData> {
        // Verify that the keys match
        if self.signed_with.key != journalist_identity_public_key.key {
            return Err(anyhow::anyhow!(
                "The signed_with key does not match the provided journalist identity public key"
            ));
        }

        // Verify the signature
        journalist_identity_public_key.verify(
            &self.backup_data_bytes,
            &self.backup_data_signature,
            now,
        )?;

        Ok(VerifiedBackupData {
            backup_data_bytes: self.backup_data_bytes,
            backup_data_signature: self.backup_data_signature,
            signed_with: journalist_identity_public_key.clone(),
        })
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct VerifiedBackupData {
    pub backup_data_bytes: BackupDataBytes,
    pub backup_data_signature: Signature<BackupDataBytes>,
    pub signed_with: SignedPublicSigningKey<JournalistId>,
}

impl VerifiedBackupData {
    pub fn to_unverified(self) -> anyhow::Result<BackupDataWithSignature> {
        let res = BackupDataWithSignature {
            backup_data_bytes: self.backup_data_bytes,
            backup_data_signature: self.backup_data_signature,
            signed_with: self.signed_with.to_untrusted(),
        };
        Ok(res)
    }

    pub fn backup_data(&self) -> anyhow::Result<BackupData> {
        BackupData::from_bytes(self.backup_data_bytes.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::{SecretBoxKey, SECRET_BOX_KEY_LEN};
    use crate::time::now;
    use rand::RngCore;

    #[test]
    fn backup_data_round_trip() -> anyhow::Result<()> {
        let backup_data = _create_sample_backup_data()?;

        let bytes = backup_data.to_bytes()?;
        let deserialized_backup_data = BackupData::from_bytes(&bytes.0)?;

        assert_eq!(backup_data, deserialized_backup_data);

        // Reserializing should yield the same bytes
        let reserialized_bytes = deserialized_backup_data.to_bytes()?;
        assert_eq!(bytes.0, reserialized_bytes.0);
        Ok(())
    }

    #[test]
    fn encrypted_secret_share_base64_round_trip() -> anyhow::Result<()> {
        let original_share = BackupEncryptedSecretShare::from_vec_unchecked(vec![1, 2, 3, 4, 5]);
        let base64_string = original_share.to_base64_string();
        let decoded_share = BackupEncryptedSecretShare::from_base64_string(&base64_string)?;
        assert_eq!(original_share, decoded_share);

        let original_share = EncryptedSecretShare::from_vec_unchecked(vec![1, 2, 3, 4, 5]);
        let base64_string = original_share.to_base64_string();
        let decoded_share = EncryptedSecretShare::from_base64_string(&base64_string)?;
        assert_eq!(original_share, decoded_share);

        Ok(())
    }

    #[test]
    fn signed_backup_data_round_trip() -> anyhow::Result<()> {
        let now = now();
        let backup_data = _create_sample_backup_data()?;

        // Self-signed key pair for testing
        let journalist_identity_key_pair: SignedSigningKeyPair<JournalistId> =
            SignedSigningKeyPair::generate()
                .to_self_signed_key_pair(now + chrono::Duration::days(30));
        let journalist_identity_public_key = journalist_identity_key_pair.public_key().clone();

        let signed_backup_data =
            backup_data.to_backup_data_with_signature(&journalist_identity_key_pair)?;
        let signed_backup_data_bytes = serde_json::to_vec(&signed_backup_data)?;

        // Happy path
        let signed_backup_data: BackupDataWithSignature =
            serde_json::from_slice(&signed_backup_data_bytes)?;
        let verified_backup_data =
            signed_backup_data.to_verified(&journalist_identity_public_key, now)?;
        let deserialized_backup_data = verified_backup_data.backup_data()?;
        assert_eq!(backup_data, deserialized_backup_data);

        // Failure path: tampered data
        let mut tampered_bytes = signed_backup_data_bytes.clone();
        // Flip a bit to simulate tampering. For the next person editing: be careful to not break
        // the JSON structure...
        tampered_bytes[32] ^= 0x01;
        let tampered_signed_backup_data: BackupDataWithSignature =
            serde_json::from_slice(&tampered_bytes)?;
        let result = tampered_signed_backup_data.to_verified(&journalist_identity_public_key, now);
        assert!(result.is_err(), "Tampered data should fail verification");

        // Failure path: expired key
        let expired_time = now + chrono::Duration::days(31);
        let signed_backup_data =
            backup_data.to_backup_data_with_signature(&journalist_identity_key_pair)?;
        let result = signed_backup_data.to_verified(&journalist_identity_public_key, expired_time);
        assert!(result.is_err(), "Expired key should fail verification");

        Ok(())
    }

    fn _create_sample_backup_data() -> Result<BackupData, Error> {
        let journalist_identity = JournalistIdentity::new("journalist_123");
        let recovery_journalist_identity = JournalistIdentity::new("journalist_456");
        let mut rng = rand::thread_rng();
        let mut vault_data = vec![0u8; 500 * 1024]; // 500 KiB of data
        rng.fill_bytes(&mut vault_data);
        let padded_vault = SteppingPaddedByteVector::new(vault_data)?;
        let backup_encrypted_padded_vault =
            SecretBox::encrypt(&SecretBoxKey::from([0u8; SECRET_BOX_KEY_LEN]), padded_vault)?;
        let backup_encrypted_share_1: BackupEncryptedSecretShare =
            AnonymousBox::from_vec_unchecked(vec![1, 2, 3]);
        let backup_encrypted_share_2: BackupEncryptedSecretShare =
            AnonymousBox::from_vec_unchecked(vec![4, 5, 6]);
        let backup_data = BackupData {
            journalist_identity: journalist_identity?,
            backup_encrypted_padded_vault,
            wrapped_encrypted_shares: vec![backup_encrypted_share_1, backup_encrypted_share_2],
            created_at: now(),
            recovery_contacts: vec![recovery_journalist_identity?],
        };
        Ok(backup_data)
    }
}
