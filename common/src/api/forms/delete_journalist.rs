use chrono::{DateTime, Utc};

use crate::{
    api::models::journalist_id::JournalistIdentity,
    form::Form,
    protocol::{keys::JournalistProvisioningKeyPair, roles::JournalistProvisioning},
};

pub type DeleteJournalistForm = Form<JournalistIdentity, JournalistProvisioning>;

impl DeleteJournalistForm {
    pub fn new(
        journalist_id: JournalistIdentity,
        signing_key_pair: &JournalistProvisioningKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        Self::new_from_form_data(journalist_id, signing_key_pair, now)
    }
}
