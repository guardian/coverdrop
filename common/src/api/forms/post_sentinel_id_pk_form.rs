use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    form::Form,
    identity_api::forms::post_rotate_sentinel_id::RotateSentinelIdPublicKeyForm,
    protocol::{keys::SentinelIdKeyPair, roles::SentinelId},
};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RotateSentinelIdPublicKeyFormBody {
    pub form: RotateSentinelIdPublicKeyForm,
}

impl RotateSentinelIdPublicKeyFormBody {
    pub fn new(form: RotateSentinelIdPublicKeyForm) -> Self {
        Self { form }
    }
}

// This form is created by Sentinel as part of the Sentinel ID key rotation process.
/// It is posted to the API which unwraps the inner RotateSentinelIdPublicKeyForm, and puts it on a queue
/// to be picked up be the Identity API which signs the new Sentinel ID public key.
pub type RotateSentinelIdPublicKeyFormForm = Form<RotateSentinelIdPublicKeyFormBody, SentinelId>;

impl RotateSentinelIdPublicKeyFormForm {
    pub fn new(
        form: RotateSentinelIdPublicKeyForm,
        signing_key_pair: &SentinelIdKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        let body = RotateSentinelIdPublicKeyFormBody::new(form);
        Self::new_from_form_data(body, signing_key_pair, now)
    }
}
