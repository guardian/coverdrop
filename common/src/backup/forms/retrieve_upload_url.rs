use chrono::{DateTime, Utc};

use crate::form::Form;
use crate::protocol::keys::JournalistIdKeyPair;
use crate::protocol::roles::JournalistId;

pub type RetrieveUploadUrlForm = Form<Vec<u8>, JournalistId>;

impl RetrieveUploadUrlForm {
    pub fn new(signing_key_pair: &JournalistIdKeyPair, now: DateTime<Utc>) -> anyhow::Result<Self> {
        // signing an empty form in order to authenticate the journalist
        Self::new_from_form_data(vec![], signing_key_pair, now)
    }
}
