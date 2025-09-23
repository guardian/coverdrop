use chrono::{DateTime, Utc};

use crate::form::Form;
use crate::protocol::backup_data::BackupDataWithSignature;
use crate::protocol::keys::JournalistIdKeyPair;
use crate::protocol::roles::JournalistId;

pub type PostBackupDataForm = Form<BackupDataWithSignature, JournalistId>;

impl PostBackupDataForm {
    pub fn new(
        signed_backup_data: BackupDataWithSignature,
        signing_key_pair: &JournalistIdKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        Self::new_from_form_data(signed_backup_data, signing_key_pair, now)
    }
}
