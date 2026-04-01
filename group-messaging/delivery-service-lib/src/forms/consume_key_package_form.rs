use chrono::{DateTime, Utc};

use common::{
    api::models::journalist_id::JournalistIdentity,
    form::Form,
    protocol::{keys::JournalistIdKeyPair, roles::JournalistId},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConsumeKeyPackageFormBody {
    pub target_client_id: JournalistIdentity,
}

/// Form for consuming a key package from a target client.
/// Used to fetch the key material needed to add a new member to an MLS group.
/// Key packages are single-use, so the delivery service marks the key package as consumed and prevents reuse.
#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct ConsumeKeyPackageForm(Form<ConsumeKeyPackageFormBody, JournalistId>);

impl ConsumeKeyPackageForm {
    pub fn new(
        target_client_id: JournalistIdentity,
        signing_key_pair: &JournalistIdKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        let body = ConsumeKeyPackageFormBody { target_client_id };
        let form = Form::new_from_form_data(body, signing_key_pair, now)?;
        Ok(Self(form))
    }
}

impl std::ops::Deref for ConsumeKeyPackageForm {
    type Target = Form<ConsumeKeyPackageFormBody, JournalistId>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
