use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    form::Form,
    protocol::{
        keys::{
            SentinelIdKeyPair, UnregisteredSentinelIdPublicKey,
            UntrustedUnregisteredSentinelIdPublicKey,
        },
        roles::SentinelId,
    },
};

// This form is provided by the API.
// A sentinel signs a form containing an unregistered ID public key and submits
// that to a "queue" of public keys for the identity-api to sign with its provisioning
// keys.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RotateSentinelIdPublicKeyBody {
    pub new_pk: UntrustedUnregisteredSentinelIdPublicKey,
}

impl RotateSentinelIdPublicKeyBody {
    pub fn new(new_pk: &UnregisteredSentinelIdPublicKey) -> Self {
        let new_pk = new_pk.to_untrusted();

        Self { new_pk }
    }
}

pub type RotateSentinelIdPublicKeyForm = Form<RotateSentinelIdPublicKeyBody, SentinelId>;

impl RotateSentinelIdPublicKeyForm {
    pub fn new(
        new_pk: &UnregisteredSentinelIdPublicKey,
        signing_key_pair: &SentinelIdKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        let body = RotateSentinelIdPublicKeyBody::new(new_pk);
        Self::new_from_form_data(body, signing_key_pair, now)
    }
}
