use crate::api::models::journalist_id::JournalistIdentity;
use crate::backup::keys::BackupIdKeyPair;
use crate::backup::roles::BackupId;
use crate::form::Form;
use chrono::{DateTime, Utc};

pub type GetBackupDataForm = Form<JournalistIdentity, BackupId>;

impl GetBackupDataForm {
    pub fn new(
        journalist_id: JournalistIdentity,
        signing_key_pair: &BackupIdKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        Self::new_from_form_data(journalist_id, signing_key_pair, now)
    }
}
