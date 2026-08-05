use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    form::Form,
    identity_api::forms::post_rotate_journalist_id::RotateJournalistIdPublicKeyForm,
    protocol::{keys::JournalistIdKeyPair, roles::JournalistId},
};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RotateJournalistIdPublicKeyFormBody {
    pub form: RotateJournalistIdPublicKeyForm,
}

impl RotateJournalistIdPublicKeyFormBody {
    pub fn new(form: RotateJournalistIdPublicKeyForm) -> Self {
        Self { form }
    }
}

// This form is created by Sentinel as part of the Journalist ID key rotation process.
/// It is posted to the API which unwraps the inner RotateJournalistIdPublicKeyForm, and puts it on a queue
/// to be picked up be the Identity API which signs the new Journalist ID public key.
pub type RotateJournalistIdPublicKeyFormForm =
    Form<RotateJournalistIdPublicKeyFormBody, JournalistId>;

impl RotateJournalistIdPublicKeyFormForm {
    pub fn new(
        form: RotateJournalistIdPublicKeyForm,
        signing_key_pair: &JournalistIdKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        let body = RotateJournalistIdPublicKeyFormBody::new(form);
        Self::new_from_form_data(body, signing_key_pair, now)
    }
}
