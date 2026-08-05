use chrono::{DateTime, Utc};

use common::{
    form::Form,
    protocol::{keys::SentinelIdKeyPair, roles::SentinelId},
};
use openmls::prelude::KeyPackageIn;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegisterClientFormBody {
    pub key_packages: Vec<KeyPackageIn>,
}

/// Form for registering a new client with the delivery service.
/// Used during initial client setup to establish the client's identity and provide initial key packages.
#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct RegisterClientForm(Form<RegisterClientFormBody, SentinelId>);

impl RegisterClientForm {
    pub fn new(
        key_packages: Vec<KeyPackageIn>,
        signing_key_pair: &SentinelIdKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        let body = RegisterClientFormBody { key_packages };
        let form = Form::new_from_form_data(body, signing_key_pair, now)?;
        Ok(Self(form))
    }
}

impl std::ops::Deref for RegisterClientForm {
    type Target = Form<RegisterClientFormBody, SentinelId>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
