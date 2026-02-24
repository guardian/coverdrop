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

// Deprecated form which does not include metadata in the presigned URL response.
// This should be removed once there are no Sentinel versions which rely on it.
pub type RetrieveUploadUrlForm = Form<Vec<u8>, JournalistId>;

impl RetrieveUploadUrlForm {
    pub fn new(signing_key_pair: &JournalistIdKeyPair, now: DateTime<Utc>) -> anyhow::Result<Self> {
        // signing an empty form in order to authenticate the journalist
        Self::new_from_form_data(vec![], signing_key_pair, now)
    }
}
