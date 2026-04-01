use chrono::{DateTime, Utc};

use common::{
    form::Form,
    protocol::{keys::JournalistIdKeyPair, roles::JournalistId},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GetClientsFormBody {
    // Empty body - this form is just for authentication
}

/// Form for retrieving a list of registered clients from the delivery service.
/// The form body is empty as its only used for authentication.
#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct GetClientsForm(Form<GetClientsFormBody, JournalistId>);

impl GetClientsForm {
    pub fn new(signing_key_pair: &JournalistIdKeyPair, now: DateTime<Utc>) -> anyhow::Result<Self> {
        let body = GetClientsFormBody {};
        let form = Form::new_from_form_data(body, signing_key_pair, now)?;
        Ok(Self(form))
    }
}

impl std::ops::Deref for GetClientsForm {
    type Target = Form<GetClientsFormBody, JournalistId>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
