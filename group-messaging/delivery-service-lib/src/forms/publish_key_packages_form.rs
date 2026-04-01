use chrono::{DateTime, Utc};

use common::{
    form::Form,
    protocol::{keys::JournalistIdKeyPair, roles::JournalistId},
};
use openmls::prelude::KeyPackageIn;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublishKeyPackagesFormBody {
    pub key_packages: Vec<KeyPackageIn>,
}

/// Form for publishing MLS key packages to the delivery service.
/// Used by clients to replenish their supply of key packages so others can start conversations with them.
#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct PublishKeyPackagesForm(Form<PublishKeyPackagesFormBody, JournalistId>);

impl PublishKeyPackagesForm {
    pub fn new(
        key_packages: Vec<KeyPackageIn>,
        signing_key_pair: &JournalistIdKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        let body = PublishKeyPackagesFormBody { key_packages };
        let form = Form::new_from_form_data(body, signing_key_pair, now)?;
        Ok(Self(form))
    }
}

impl std::ops::Deref for PublishKeyPackagesForm {
    type Target = Form<PublishKeyPackagesFormBody, JournalistId>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
