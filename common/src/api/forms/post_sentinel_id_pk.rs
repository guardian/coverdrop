use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    api::models::sentinel_id::SentinelIdentity,
    form::Form,
    protocol::{
        keys::{JournalistProvisioningKeyPair, UntrustedSentinelIdPublicKey},
        roles::JournalistProvisioning,
    },
};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostSentinelIdPublicKeyBody {
    pub sentinel_id: SentinelIdentity,
    pub sentinel_id_pk: UntrustedSentinelIdPublicKey,
    /// from_queue is true if the form is being submitted by the identity API as part of an automatic id key rotation,
    /// and false if it is being submitted by an admin registering the sentinel profile for the first time.
    /// This form is signed by the provisioning key and we trust the identity API to submit the correct value.
    pub from_queue: bool,
}

pub type PostSentinelIdPublicKeyForm = Form<PostSentinelIdPublicKeyBody, JournalistProvisioning>;

impl PostSentinelIdPublicKeyForm {
    pub fn new(
        sentinel_id: SentinelIdentity,
        sentinel_id_pk: UntrustedSentinelIdPublicKey,
        from_queue: bool,
        signing_key_pair: &JournalistProvisioningKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        let body = PostSentinelIdPublicKeyBody {
            sentinel_id,
            sentinel_id_pk,
            from_queue,
        };

        Self::new_from_form_data(body, signing_key_pair, now)
    }
}
