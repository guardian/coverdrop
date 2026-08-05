use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    api::models::sentinel_id::SentinelIdentity,
    form::Form,
    protocol::{keys::JournalistProvisioningKeyPair, roles::JournalistProvisioning},
};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostSentinelProfileBody {
    pub id: SentinelIdentity,
    pub display_name: String,
}

pub type PostSentinelProfileForm = Form<PostSentinelProfileBody, JournalistProvisioning>;

impl PostSentinelProfileForm {
    pub fn new(
        id: SentinelIdentity,
        display_name: String,
        signing_key_pair: &JournalistProvisioningKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        let body = PostSentinelProfileBody { id, display_name };
        Self::new_from_form_data(body, signing_key_pair, now)
    }
}
