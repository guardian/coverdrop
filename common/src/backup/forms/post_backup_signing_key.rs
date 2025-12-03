use crate::backup::keys::UntrustedBackupIdPublicKey;
use crate::form::Form;
use crate::protocol::keys::OrganizationKeyPair;
use crate::protocol::roles::Organization;
use chrono::{DateTime, Utc};

pub const BACKUP_SIGNING_KEY_FORM_FILENAME: &str = "backup_identity_key_form.json";

pub type PostBackupIdKeyForm = Form<UntrustedBackupIdPublicKey, Organization>;

impl PostBackupIdKeyForm {
    pub fn new(
        backup_signing_pk: UntrustedBackupIdPublicKey,
        signing_key_pair: &OrganizationKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        Self::new_from_form_data(backup_signing_pk, signing_key_pair, now)
    }
}
