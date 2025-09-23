use crate::api::models::journalist_id::JournalistIdentity;
use crate::backup::roles::BackupMsg;
use crate::crypto::keys::encryption::{SignedEncryptionKeyPair, SignedPublicEncryptionKey};
use crate::crypto::keys::signing::{SignedPublicSigningKey, SignedSigningKeyPair};
use crate::crypto::{
    AnonymousBox, SecretBox, SecretSharingScheme, SecretSharingSecret, SecretSharingShare,
    SingleShareSecretSharing,
};
use crate::padded_byte_vector::SteppingPaddedByteVector;
use crate::protocol::backup_data::{
    BackupData, BackupDataWithSignature, BackupEncryptedPaddedVault, BackupEncryptedSecretShare,
    EncryptedSecretShare, VerifiedBackupData,
};
use crate::protocol::roles::{JournalistId, JournalistMessaging};
use anyhow::Context;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// A recovery contact for a Sentinel backup, consisting of their identity and latest messaging key.
#[derive(Clone, Debug)]
pub struct RecoveryContact {
    pub identity: JournalistIdentity,
    pub latest_messaging_key: SignedPublicEncryptionKey<JournalistMessaging>,
}

/// Runs on the journalist's device to create a Sentinel backup. It encrypts the provided
/// `encrypted_vault` (which is always encrypted on disk) under a fresh ephemeral symmetric key,
/// splits that key into `n` shares using a (k,n) secret sharing scheme, encrypts each share
/// under a recovery contact's latest messaging key, and then wraps each encrypted share under
/// the backup admin's encryption key. The resulting backup data is signed using the journalist's
/// identity key and returned.
///
/// `k` must be 1 for now, meaning that any single recovery contact can help restore the backup.
/// `n` is determined by the number of provided recovery contacts, and must be at least `k`.
pub fn sentinel_create_backup(
    encrypted_vault: Vec<u8>,
    journalist_identity: JournalistIdentity,
    journalist_identity_key: SignedSigningKeyPair<JournalistId>,
    backup_admin_encryption_key: SignedPublicEncryptionKey<BackupMsg>,
    recovery_contacts: Vec<RecoveryContact>,
    k: usize,
    now: DateTime<Utc>,
) -> anyhow::Result<VerifiedBackupData> {
    let n = recovery_contacts.len();

    if k != 1 {
        return Err(anyhow::anyhow!(
            "The backup protocol only supports k=1, but got k={k}"
        ));
    }
    if n < k {
        return Err(anyhow::anyhow!(
            "Cannot create backup with {n} recovery contacts, at least {k} are required"
        ));
    }

    // Ephemeral symmetric key that encrypts the vault (and then gets split into shares)
    let sk = SecretSharingSecret::generate()?;

    // Pad the vault to hide its precise size and encrypt under the ephemeral key `sk`
    let padded_encrypted_vault = SteppingPaddedByteVector::new(encrypted_vault)
        .map_err(|e| anyhow::anyhow!("Failed to pad encrypted vault: {}", e))?;
    let backup_encrypted_padded_vault =
        SecretBox::encrypt(sk.as_bytes().into(), padded_encrypted_vault)
            .map_err(|e| anyhow::anyhow!("Failed to encrypt padded vault: {}", e))?;

    // Split the ephemeral key into `k` shares
    let shares = SingleShareSecretSharing::split(sk, n)?;

    // Encrypt each share under a recovery contact's messaging key; for this, zip them together
    if shares.len() != recovery_contacts.len() {
        return Err(anyhow::anyhow!(
            "Number of shares ({}) does not match number of recovery contacts ({})",
            shares.len(),
            recovery_contacts.len()
        ));
    }
    let encrypted_shares: Vec<EncryptedSecretShare> = shares
        .into_iter()
        .zip(recovery_contacts)
        .map(|(share, contact)| {
            let key = contact.latest_messaging_key.to_public_encryption_key();
            AnonymousBox::encrypt(&key, share).context("Failed to encrypt share")
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Encrypt each share under the backup admin encryption key
    let wrapped_encrypted_shares: Vec<BackupEncryptedSecretShare> = encrypted_shares
        .into_iter()
        .map(|encrypted_share| {
            let key = backup_admin_encryption_key
                .clone()
                .to_public_encryption_key();
            AnonymousBox::encrypt(&key, encrypted_share).context("Failed to wrap encrypted share")
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Create the backup data and sign it
    let backup_data = BackupData {
        journalist_identity,
        backup_encrypted_padded_vault,
        wrapped_encrypted_shares,
        created_at: now,
    };
    let backup_data_with_signature = backup_data
        .to_backup_data_with_signature(&journalist_identity_key)
        .context("Failed to generate backup data")?;
    let verified_backup_data = backup_data_with_signature
        .to_verified(journalist_identity_key.public_key(), now)
        .context("Failed to freshly-created verify backup data")?;

    Ok(verified_backup_data)
}

/// The initial state during the recovery process after the backup admin has retrieved the
/// backup data and has unwrapped the encrypted shares.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BackupRestorationInProgress {
    journalist_identity: JournalistIdentity,
    #[serde(with = "BackupEncryptedPaddedVault")]
    backup_encrypted_padded_vault: BackupEncryptedPaddedVault,
    pub encrypted_shares: Vec<EncryptedSecretShare>,
}

/// Runs on the backup admin's device to initiate the restoration of a Sentinel backup. It
/// verifies the provided signed backup data using the journalist's identity public key, unwraps
/// the encrypted shares using the backup admin's encryption key pair, and returns the initial
/// state for the restoration process.
///
/// The integrating application should store the `BackupRestorationInProgress` state on
/// disk and distribute the `encrypted_shares` to the recovery contacts, asking them to
/// attempt to decrypt them using their messaging keys (see
/// `sentinel_restore_try_unwrap_share_step`).
pub fn coverup_initiate_restore_step(
    journalist_identity: JournalistIdentity,
    signed_backup_data: BackupDataWithSignature,
    journalist_identity_public_key: &SignedPublicSigningKey<JournalistId>,
    backup_admin_encryption_key_pair: &SignedEncryptionKeyPair<BackupMsg>,
    now: DateTime<Utc>,
) -> anyhow::Result<BackupRestorationInProgress> {
    // Retrieve and verify the backup data
    let verified_backup_data = signed_backup_data
        .to_verified(journalist_identity_public_key, now)
        .context("Failed to verify backup data signature")?;
    let backup_data = verified_backup_data.backup_data()?;

    if backup_data.journalist_identity != journalist_identity {
        return Err(anyhow::anyhow!(
            "The backup's journalist identity does not match the provided identity"
        ));
    }

    // Remove the outer layer of encryption from the shares
    let unwrapped_encrypted_shares: Vec<EncryptedSecretShare> = backup_data
        .wrapped_encrypted_shares
        .iter()
        .map(|wrapped_share| {
            AnonymousBox::decrypt(backup_admin_encryption_key_pair, wrapped_share)
                .map_err(|e| anyhow::anyhow!("Failed to unwrap encrypted share: {}", e))
        })
        .collect::<Result<Vec<_>, _>>()?;

    if unwrapped_encrypted_shares.is_empty() {
        return Err(anyhow::anyhow!(
            "No encrypted shares found in the backup data"
        ));
    }

    let backup_state = BackupRestorationInProgress {
        journalist_identity: backup_data.journalist_identity,
        backup_encrypted_padded_vault: backup_data.backup_encrypted_padded_vault,
        encrypted_shares: unwrapped_encrypted_shares,
    };
    Ok(backup_state)
}

/// A secret share that is encrypted under the backup admin's encryption key.
type WrappedSecretShare = AnonymousBox<SecretSharingShare>;

/// Runs on a recovery contact's device to attempt to decrypt any of the provided
/// `encrypted_share_candidates` using their own messaging key pairs. If successful, the decrypted
/// share is then wrapped under the backup admin's encryption key for transport back to them.
/// If none of the provided keys can decrypt any of the shares, `Ok(None)` is returned.
pub fn sentinel_restore_try_unwrap_share_step(
    encrypted_share_candidates: Vec<EncryptedSecretShare>,
    recovery_contact_messaging_key_pairs: Vec<SignedEncryptionKeyPair<JournalistMessaging>>,
    backup_admin_encryption_key: SignedPublicEncryptionKey<BackupMsg>,
) -> anyhow::Result<Option<WrappedSecretShare>> {
    let admin_key = backup_admin_encryption_key.to_public_encryption_key();
    for key_pair in recovery_contact_messaging_key_pairs.iter() {
        for encrypted_share in encrypted_share_candidates.iter() {
            if let Ok(share) = AnonymousBox::decrypt(key_pair, encrypted_share) {
                // Encrypt under the backup admin's key for transport back to them
                let wrapped_share = AnonymousBox::encrypt(&admin_key, share)
                    .map_err(|e| anyhow::anyhow!("Failed to wrap decrypted share: {}", e))?;
                return Ok(Some(wrapped_share));
            }
        }
    }

    // None of the provided keys could decrypt any of the shares
    Ok(None)
}

/// Runs on the backup admin's device to complete the restoration of a Sentinel backup. It
/// attempts to decrypt the provided `wrapped_shares` using their encryption key pair, combines
/// the resulting shares to reconstruct the ephemeral symmetric key, and then decrypts the
/// backup vault.
///
/// The integrating application should have previously stored the `BackupRestorationInProgress`
/// state on disk after calling `coverup_initiate_restore_step`, and should provide it here.
/// If successful, the encrypted vault is returned and can be persisted on disk.
pub fn sentinel_finish_restore_step(
    backup_state: BackupRestorationInProgress,
    wrapped_shares: Vec<WrappedSecretShare>,
    backup_admin_encryption_key_pair: &SignedEncryptionKeyPair<BackupMsg>,
) -> anyhow::Result<Vec<u8>> {
    if wrapped_shares.is_empty() {
        return Err(anyhow::anyhow!("No wrapped shares provided"));
    }

    // Unwrap the shares using the backup admin's encryption key pair
    let unwrapped_shares: Vec<SecretSharingShare> = wrapped_shares
        .iter()
        .map(|wrapped_share| {
            AnonymousBox::decrypt(backup_admin_encryption_key_pair, wrapped_share)
                .map_err(|e| anyhow::anyhow!("Failed to unwrap share: {}", e))
        })
        .collect::<Result<Vec<_>, _>>()?;

    if unwrapped_shares.is_empty() {
        return Err(anyhow::anyhow!("No unwrapped shares available"));
    }

    // For now, we only support k=1, so we need exactly one share to reconstruct the key
    let unwrapped_shares: [SecretSharingShare; 1] = unwrapped_shares
        .into_iter()
        .take(1)
        .collect::<Vec<_>>()
        .try_into()
        .map_err(|_| anyhow::anyhow!("Failed to collect exactly one unwrapped share"))?;

    // Combine the shares to reconstruct the ephemeral symmetric key
    let sk = SingleShareSecretSharing::combine(unwrapped_shares)
        .map_err(|e| anyhow::anyhow!("Failed to combine shares: {}", e))?;

    // Decrypt the padded vault using the reconstructed key
    let padded_encrypted_vault = SecretBox::decrypt(
        sk.as_bytes().into(),
        backup_state.backup_encrypted_padded_vault,
    )
    .map_err(|e| anyhow::anyhow!("Failed to decrypt padded vault: {}", e))?;

    // Remove the padding to retrieve the original encrypted vault
    let encrypted_vault = padded_encrypted_vault
        .into_unpadded()
        .map_err(|e| anyhow::anyhow!("Failed to unpad decrypted vault: {}", e))?;

    Ok(encrypted_vault)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::models::journalist_id::JournalistIdentity;
    use crate::crypto::keys::encryption::{SignedEncryptionKeyPair, UnsignedEncryptionKeyPair};
    use crate::crypto::keys::signing::UnsignedSigningKeyPair;
    use crate::protocol::roles::{JournalistId, JournalistMessaging, JournalistProvisioning};
    use crate::time::now;

    fn create_test_journalist_identity(identifier: String) -> anyhow::Result<JournalistIdentity> {
        JournalistIdentity::new(identifier.as_str()).map_err(|e| anyhow::anyhow!(e))
    }

    fn create_test_journalist_signing_key_pair() -> SignedSigningKeyPair<JournalistId> {
        let unsigned_pair = UnsignedSigningKeyPair::generate();
        let not_valid_after = now() + chrono::Duration::days(30);
        // self-signed for testing
        unsigned_pair
            .clone()
            .to_signed_key_pair(&unsigned_pair, not_valid_after)
    }

    fn create_test_journalist_messaging_key_pair(
        signed_signing_key_pair: &SignedSigningKeyPair<JournalistId>,
    ) -> SignedEncryptionKeyPair<JournalistMessaging> {
        let unsigned_pair = UnsignedEncryptionKeyPair::generate();
        let not_valid_after = now() + chrono::Duration::days(30);
        unsigned_pair.to_signed_key_pair(&signed_signing_key_pair, not_valid_after)
    }

    fn create_test_journalist(
        identifier: String,
    ) -> anyhow::Result<(
        JournalistIdentity,
        SignedSigningKeyPair<JournalistId>,
        SignedEncryptionKeyPair<JournalistMessaging>,
    )> {
        let identity = create_test_journalist_identity(identifier)?;
        let signing_key_pair = create_test_journalist_signing_key_pair();
        let messaging_key_pair = create_test_journalist_messaging_key_pair(&signing_key_pair);
        Ok((identity, signing_key_pair, messaging_key_pair))
    }

    fn create_test_backup_admin_encryption_key_pair() -> SignedEncryptionKeyPair<BackupMsg> {
        let unsigned_signing_pair: UnsignedSigningKeyPair<JournalistProvisioning> =
            UnsignedSigningKeyPair::generate();
        let not_valid_after = now() + chrono::Duration::days(30);
        // self-signed for testing
        let signed_signing_key_pair = unsigned_signing_pair
            .clone()
            .to_signed_key_pair(&unsigned_signing_pair, not_valid_after);
        let unsigned_encryption_pair = UnsignedEncryptionKeyPair::generate();
        let not_valid_after = now() + chrono::Duration::days(30);

        unsigned_encryption_pair.to_signed_key_pair(&signed_signing_key_pair, not_valid_after)
    }

    fn create_test_vault_data() -> Vec<u8> {
        b"test encrypted vault data".to_vec()
    }

    #[test]
    fn test_round_trip_backup_and_restore_k1() -> anyhow::Result<()> {
        // Create test data
        let (journalist_identity, journalist_signing_pair, _) =
            create_test_journalist("journalist1".to_string())?;
        let backup_admin_encryption_pair = create_test_backup_admin_encryption_key_pair();
        let (_, _, recovery_contact_messaging_pair) =
            create_test_journalist("recovery_contact1".to_string())?;
        let encrypted_vault = create_test_vault_data();

        // Create recovery contact
        let recovery_contact = RecoveryContact {
            identity: journalist_identity.clone(),
            latest_messaging_key: recovery_contact_messaging_pair.public_key().clone(),
        };

        // Step 1: Create backup
        let signed_backup_data = sentinel_create_backup(
            encrypted_vault.clone(),
            journalist_identity.clone(),
            journalist_signing_pair.clone(),
            backup_admin_encryption_pair.public_key().clone(),
            vec![recovery_contact],
            1, // k=1
            now(),
        )
        .expect("Failed to create backup");

        // Step 2: Initiate restore
        let backup_state = coverup_initiate_restore_step(
            journalist_identity.clone(),
            signed_backup_data.to_unverified()?,
            &journalist_signing_pair.public_key(),
            &backup_admin_encryption_pair,
            now(),
        )
        .expect("Failed to initiate restore");

        // Step 3: Recovery contact unwraps share
        let wrapped_share = sentinel_restore_try_unwrap_share_step(
            backup_state.encrypted_shares.clone(),
            vec![recovery_contact_messaging_pair],
            backup_admin_encryption_pair.public_key().clone(),
        )
        .expect("Failed to unwrap share")
        .expect("No share could be unwrapped");

        // Step 4: Complete restore
        let restored_vault = sentinel_finish_restore_step(
            backup_state,
            vec![wrapped_share],
            &backup_admin_encryption_pair,
        )
        .expect("Failed to finish restore");

        // Verify the round-trip worked
        assert_eq!(encrypted_vault, restored_vault);

        Ok(())
    }

    #[test]
    fn test_tampered_wrapped_encrypted_secret_shares_fail() -> anyhow::Result<()> {
        // Create test data
        let (journalist_identity, journalist_signing_pair, _) =
            create_test_journalist("journalist1".to_string())?;
        let backup_admin_encryption_pair = create_test_backup_admin_encryption_key_pair();
        let (_, _, recovery_contact_messaging_pair) =
            create_test_journalist("recovery_contact1".to_string())?;
        let encrypted_vault = create_test_vault_data();

        let recovery_contact = RecoveryContact {
            identity: journalist_identity.clone(),
            latest_messaging_key: recovery_contact_messaging_pair.public_key().clone(),
        };

        // Create backup
        let signed_backup_data = sentinel_create_backup(
            encrypted_vault,
            journalist_identity.clone(),
            journalist_signing_pair.clone(),
            backup_admin_encryption_pair.public_key().clone(),
            vec![recovery_contact],
            1,
            now(),
        )
        .expect("Failed to create backup");

        // Tamper with the wrapped encrypted secret shares; for this first deserialize
        // the backup data, modify it, and then serialize it back
        let mut backup_data = signed_backup_data.backup_data()?;
        let first_share_bytes = &mut backup_data.wrapped_encrypted_shares[0].as_bytes().to_vec();
        first_share_bytes[0] ^= 0x01; // Flip a bit to simulate tampering
        backup_data.wrapped_encrypted_shares[0] =
            AnonymousBox::from_vec_unchecked(first_share_bytes.clone());

        // Re-pack and re-sign the tampered data
        let signed_backup_data =
            backup_data.to_backup_data_with_signature(&journalist_signing_pair)?;

        // Attempt to restore - should fail during initiation
        let result = coverup_initiate_restore_step(
            journalist_identity,
            signed_backup_data,
            &journalist_signing_pair.public_key(),
            &backup_admin_encryption_pair,
            now(),
        );

        assert!(
            result.is_err(),
            "Restore should fail with tampered wrapped shares"
        );

        Ok(())
    }

    #[test]
    fn test_different_backup_admin_key_fails() -> anyhow::Result<()> {
        // Create test data
        let (journalist_identity, journalist_signing_pair, _) =
            create_test_journalist("journalist1".to_string())?;
        let backup_admin_encryption_pair = create_test_backup_admin_encryption_key_pair();
        let different_backup_admin_pair = create_test_backup_admin_encryption_key_pair();
        let (_, _, recovery_contact_messaging_pair) =
            create_test_journalist("recovery_contact1".to_string())?;
        let encrypted_vault = create_test_vault_data();

        let recovery_contact = RecoveryContact {
            identity: journalist_identity.clone(),
            latest_messaging_key: recovery_contact_messaging_pair.public_key().clone(),
        };

        // Create backup with original admin key
        let signed_backup_data = sentinel_create_backup(
            encrypted_vault,
            journalist_identity.clone(),
            journalist_signing_pair.clone(),
            backup_admin_encryption_pair.public_key().clone(),
            vec![recovery_contact],
            1,
            now(),
        )
        .expect("Failed to create backup");

        // Attempt to restore with different admin key - should fail
        let result = coverup_initiate_restore_step(
            journalist_identity,
            signed_backup_data.to_unverified()?,
            &journalist_signing_pair.public_key(),
            &different_backup_admin_pair, // Different key!
            now(),
        );

        assert!(
            result.is_err(),
            "Restore should fail with different backup admin key"
        );

        Ok(())
    }

    #[test]
    fn test_tampered_signed_backup_data_fails() -> anyhow::Result<()> {
        // Create test data
        let (journalist_identity, journalist_signing_pair, _) =
            create_test_journalist("journalist1".to_string())?;
        let backup_admin_encryption_pair = create_test_backup_admin_encryption_key_pair();
        let (_, _, recovery_contact_messaging_pair) =
            create_test_journalist("recovery_contact1".to_string())?;
        let encrypted_vault = create_test_vault_data();

        let recovery_contact = RecoveryContact {
            identity: journalist_identity.clone(),
            latest_messaging_key: recovery_contact_messaging_pair.public_key().clone(),
        };

        // Create backup
        let verified_backup_data = sentinel_create_backup(
            encrypted_vault,
            journalist_identity.clone(),
            journalist_signing_pair.clone(),
            backup_admin_encryption_pair.public_key().clone(),
            vec![recovery_contact],
            1,
            now(),
        )
        .expect("Failed to create backup");

        // Tamper with the encrypted vault
        let mut backup_data = verified_backup_data.backup_data()?;
        let backup_encrypted_vault_bytes = &mut backup_data
            .backup_encrypted_padded_vault
            .as_bytes()
            .to_vec();
        backup_encrypted_vault_bytes[0] ^= 0x01; // Flip a bit to simulate tampering
        backup_data.backup_encrypted_padded_vault =
            SecretBox::from_vec_unchecked(backup_encrypted_vault_bytes.clone());

        // Re-pack, but do not resign
        let tampered_backup_data_with_signature = BackupDataWithSignature::new(
            backup_data.to_bytes()?,
            verified_backup_data.backup_data_signature.clone(),
            verified_backup_data.signed_with.to_untrusted(),
        )?;

        // Attempt to restore - should fail during signature verification
        let result = coverup_initiate_restore_step(
            journalist_identity,
            tampered_backup_data_with_signature,
            &journalist_signing_pair.public_key(),
            &backup_admin_encryption_pair,
            now(),
        );

        assert!(
            result.is_err(),
            "Restore should fail with tampered signed backup data"
        );

        Ok(())
    }

    #[test]
    fn test_tampered_encrypted_vault_fails() -> anyhow::Result<()> {
        // Create test data
        let (journalist_identity, journalist_signing_pair, _) =
            create_test_journalist("journalist1".to_string())?;
        let backup_admin_encryption_pair = create_test_backup_admin_encryption_key_pair();
        let (_, _, recovery_contact_messaging_pair) =
            create_test_journalist("recovery_contact1".to_string())?;
        let encrypted_vault = create_test_vault_data();

        let recovery_contact = RecoveryContact {
            identity: journalist_identity.clone(),
            latest_messaging_key: recovery_contact_messaging_pair.public_key().clone(),
        };

        // Create backup
        let verified_backup_data = sentinel_create_backup(
            encrypted_vault,
            journalist_identity.clone(),
            journalist_signing_pair.clone(),
            backup_admin_encryption_pair.public_key().clone(),
            vec![recovery_contact],
            1,
            now(),
        )
        .expect("Failed to create backup");

        // Tamper with the encrypted vault
        let mut backup_data = verified_backup_data.backup_data()?;
        let backup_encrypted_vault_bytes = &mut backup_data
            .backup_encrypted_padded_vault
            .as_bytes()
            .to_vec();
        backup_encrypted_vault_bytes[0] ^= 0x01; // Flip a bit to simulate tampering
        backup_data.backup_encrypted_padded_vault =
            SecretBox::from_vec_unchecked(backup_encrypted_vault_bytes.clone());

        // Re-pack and re-sign the tampered data
        let signed_backup_data =
            backup_data.to_backup_data_with_signature(&journalist_signing_pair)?;

        // Initiate restore (should succeed since signature is valid)
        let backup_state = coverup_initiate_restore_step(
            journalist_identity,
            signed_backup_data,
            &journalist_signing_pair.public_key(),
            &backup_admin_encryption_pair,
            now(),
        )
        .expect("Failed to initiate restore");

        // Recovery contact unwraps share
        let wrapped_share = sentinel_restore_try_unwrap_share_step(
            backup_state.encrypted_shares.clone(),
            vec![recovery_contact_messaging_pair],
            backup_admin_encryption_pair.public_key().clone(),
        )
        .expect("Failed to unwrap share")
        .expect("No share could be unwrapped");

        // Complete restore - should fail during vault decryption
        let result = sentinel_finish_restore_step(
            backup_state,
            vec![wrapped_share],
            &backup_admin_encryption_pair,
        );

        assert!(
            result.is_err(),
            "Restore should fail with tampered encrypted vault"
        );

        Ok(())
    }

    #[test]
    fn test_tampered_wrapped_secret_shares_before_final_step_fails() -> anyhow::Result<()> {
        // Create test data
        let (journalist_identity, journalist_signing_pair, _) =
            create_test_journalist("journalist1".to_string())?;
        let backup_admin_encryption_pair = create_test_backup_admin_encryption_key_pair();
        let (_, _, recovery_contact_messaging_pair) =
            create_test_journalist("recovery_contact1".to_string())?;
        let encrypted_vault = create_test_vault_data();

        let recovery_contact = RecoveryContact {
            identity: journalist_identity.clone(),
            latest_messaging_key: recovery_contact_messaging_pair.public_key().clone(),
        };

        // Create backup and initiate restore
        let verified_backup_data = sentinel_create_backup(
            encrypted_vault,
            journalist_identity.clone(),
            journalist_signing_pair.clone(),
            backup_admin_encryption_pair.public_key().clone(),
            vec![recovery_contact],
            1,
            now(),
        )
        .expect("Failed to create backup");

        let backup_state = coverup_initiate_restore_step(
            journalist_identity,
            verified_backup_data.to_unverified()?,
            &journalist_signing_pair.public_key(),
            &backup_admin_encryption_pair,
            now(),
        )
        .expect("Failed to initiate restore");

        // Recovery contact unwraps share
        let wrapped_share = sentinel_restore_try_unwrap_share_step(
            backup_state.encrypted_shares.clone(),
            vec![recovery_contact_messaging_pair],
            backup_admin_encryption_pair.public_key().clone(),
        )
        .expect("Failed to unwrap share")
        .expect("No share could be unwrapped");

        // Tamper with the wrapped share before final step
        let wrapped_share_bytes = &mut wrapped_share.as_bytes().to_vec();
        wrapped_share_bytes[0] ^= 0x01; // Flip a bit to simulate tampering
        let wrapped_share = AnonymousBox::from_vec_unchecked(wrapped_share_bytes.clone());

        // Complete restore - should fail during share unwrapping
        let result = sentinel_finish_restore_step(
            backup_state,
            vec![wrapped_share],
            &backup_admin_encryption_pair,
        );

        assert!(
            result.is_err(),
            "Restore should fail with tampered wrapped shares"
        );

        Ok(())
    }

    #[test]
    fn test_journalist_identity_verification() -> anyhow::Result<()> {
        // Create test data
        let (journalist_identity, journalist_signing_pair, _) =
            create_test_journalist("journalist1".to_string())?;
        let different_identity =
            create_test_journalist_identity("different-journalist".to_string())?;
        let backup_admin_encryption_pair = create_test_backup_admin_encryption_key_pair();
        let (_, _, recovery_contact_messaging_pair) =
            create_test_journalist("recovery_contact1".to_string())?;
        let encrypted_vault = create_test_vault_data();

        let recovery_contact = RecoveryContact {
            identity: journalist_identity.clone(),
            latest_messaging_key: recovery_contact_messaging_pair.public_key().clone(),
        };

        // Create backup with one identity
        let signed_backup_data = sentinel_create_backup(
            encrypted_vault,
            journalist_identity,
            journalist_signing_pair.clone(),
            backup_admin_encryption_pair.public_key().clone(),
            vec![recovery_contact],
            1,
            now(),
        )
        .expect("Failed to create backup");

        // Attempt to restore with different identity - should fail
        let result = coverup_initiate_restore_step(
            different_identity, // Different identity!
            signed_backup_data.to_unverified()?,
            &journalist_signing_pair.public_key(),
            &backup_admin_encryption_pair,
            now(),
        );

        assert!(
            result.is_err(),
            "Restore should fail with different journalist identity"
        );
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("journalist identity does not match"));

        Ok(())
    }
}
