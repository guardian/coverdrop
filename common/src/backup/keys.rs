use crate::backup::constants::{BACKUP_ID_KEY_VALID_DURATION, BACKUP_MSG_KEY_VALID_DURATION};

use crate::backup::roles::{BackupId, BackupMsg};
use crate::crypto::keys::encryption::{
    EncryptionKeyPair, PublicEncryptionKey, SignedEncryptionKeyPair, SignedPublicEncryptionKey,
    UnsignedEncryptionKeyPair,
};
use crate::crypto::keys::serde::StorableKeyMaterial;
use crate::crypto::keys::signing::{
    traits, SignedPublicSigningKey, SignedSigningKeyPair, UnsignedSigningKeyPair,
};
use crate::crypto::keys::untrusted::encryption::{
    UntrustedSignedEncryptionKeyPair, UntrustedSignedPublicEncryptionKey,
};
use crate::crypto::keys::untrusted::signing::{
    UntrustedSignedPublicSigningKey, UntrustedSignedSigningKeyPair,
};
use crate::protocol::keys::{OrganizationKeyPair, OrganizationPublicKey};
use crate::protocol::roles::AnchorOrganization;
use chrono::{DateTime, Utc};
use std::path::Path;

pub type BackupIdPublicKey = SignedPublicSigningKey<BackupId>;
pub type BackupIdKeyPair = SignedSigningKeyPair<BackupId>;

pub type UntrustedBackupIdPublicKey = UntrustedSignedPublicSigningKey<BackupId>;
pub type UntrustedBackupIdKeyPair = UntrustedSignedSigningKeyPair<BackupId>;

pub type BackupMsgPublicKey = SignedPublicEncryptionKey<BackupMsg>;
pub type BackupMsgKeyPair = SignedEncryptionKeyPair<BackupMsg>;

pub type UntrustedBackupMsgPublicKey = UntrustedSignedPublicEncryptionKey<BackupMsg>;
pub type UntrustedBackupMsgKeyPair = UntrustedSignedEncryptionKeyPair<BackupMsg>;

pub fn generate_backup_id_key_pair(
    org_key_pair: &OrganizationKeyPair,
    now: DateTime<Utc>,
) -> BackupIdKeyPair {
    let not_valid_after = now + BACKUP_ID_KEY_VALID_DURATION;

    UnsignedSigningKeyPair::generate().to_signed_key_pair(org_key_pair, not_valid_after)
}

pub fn verify_backup_id_pk(
    untrusted: &UntrustedBackupIdPublicKey,
    org_pk: &OrganizationPublicKey,
    now: DateTime<Utc>,
) -> anyhow::Result<BackupIdPublicKey> {
    untrusted.to_trusted(org_pk, now)
}

pub fn generate_backup_msg_key_pair(
    backup_id_pk: &BackupIdKeyPair,
    now: DateTime<Utc>,
) -> BackupMsgKeyPair {
    let not_valid_after = now + BACKUP_MSG_KEY_VALID_DURATION;
    let encryption_key_pair: EncryptionKeyPair<BackupMsg, PublicEncryptionKey<BackupMsg>> =
        UnsignedEncryptionKeyPair::<BackupMsg>::generate();

    encryption_key_pair.to_signed_key_pair(backup_id_pk, not_valid_after)
}

pub fn verify_backup_msg_pk(
    untrusted: &UntrustedBackupMsgPublicKey,
    backup_signing_pk: &BackupIdPublicKey,
    now: DateTime<Utc>,
) -> anyhow::Result<BackupMsgPublicKey> {
    Ok(untrusted.to_trusted(backup_signing_pk, now)?)
}

pub fn load_backup_signing_key_pair(
    keys_path: impl AsRef<Path>,
    org_pks: &[impl traits::PublicSigningKey<AnchorOrganization>],
    now: DateTime<Utc>,
) -> anyhow::Result<Vec<BackupIdKeyPair>> {
    let key_pair = UntrustedBackupIdKeyPair::load_from_directory(&keys_path)?
        .iter()
        .flat_map(|key_pair| {
            org_pks
                .iter()
                .flat_map(|org_pk| key_pair.to_trusted(org_pk, now))
        })
        .collect::<Vec<_>>();

    Ok(key_pair)
}
