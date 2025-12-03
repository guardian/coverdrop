use crate::backup::keys::BackupIdKeyPair;
use crate::backup::{keys::UntrustedBackupMsgPublicKey, roles::BackupId};
use crate::form::Form;
use chrono::{DateTime, Utc};

pub const BACKUP_MESSAGING_KEY_FORM_FILENAME: &str = "backup_messaging_key_form.json";

pub type PostBackupMsgKeyForm = Form<UntrustedBackupMsgPublicKey, BackupId>;

impl PostBackupMsgKeyForm {
    pub fn new(
        backup_encryption_pk: UntrustedBackupMsgPublicKey,
        signing_key_pair: &BackupIdKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        Self::new_from_form_data(backup_encryption_pk, signing_key_pair, now)
    }
}
