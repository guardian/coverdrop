use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::crypto::Signature;
use crate::form::Form;
use crate::protocol::backup_data::BackupDataBytes;
use crate::protocol::keys::JournalistIdKeyPair;
use crate::protocol::roles::JournalistId;

#[derive(Serialize, Deserialize)]
pub struct RetrieveUploadUrlWithMetadataFormBody {
    pub backup_data_signature: Signature<BackupDataBytes>,
}

pub type RetrieveUploadUrlWithMetadataForm =
    Form<RetrieveUploadUrlWithMetadataFormBody, JournalistId>;

impl RetrieveUploadUrlWithMetadataForm {
    pub fn new(
        backup_data_signature: Signature<BackupDataBytes>,
        signing_key_pair: &JournalistIdKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        let form_body = RetrieveUploadUrlWithMetadataFormBody {
            backup_data_signature,
        };

        Self::new_from_form_data(form_body, signing_key_pair, now)
    }
}
