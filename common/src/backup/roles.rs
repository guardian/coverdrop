use crate::{
    backup::constants::{BACKUP_ID_KEY_VALID_DURATION, BACKUP_MSG_KEY_VALID_DURATION},
    crypto::keys::id_key_certificate_data::IdKeyCertificateData,
    define_role,
};

// These two roles are here so that we can create a signing and an encryption key for journalist vault backups
// that fit easily into our key hierarchy types.
// Backup Id keys aren't really associated with an identity in the same way that journalist, covernode,
// and sentinel id keys are, and backup messaging keys don't really "belong" to an identity in the
// same way that journalist, covernode, and sentinel messaging keys do.
//
// TODO It might be better to model "backup signing" and "backup encryption" keys independently of "identity"
// and "messaging" roles as they don't follow the same patterns as the other id and messaging keys in terms of
// their validity duration, rotation period etc. Both keys could be signed directly by the organization key.

define_role!(
    BackupMsg,
    "backup message",
    "backup_msg",
    Some(BACKUP_MSG_KEY_VALID_DURATION),
    None
);

define_role!(
    BackupId,
    "backup id",
    "backup_id",
    Some(BACKUP_ID_KEY_VALID_DURATION),
    None,
    IdKeyCertificateData
);
