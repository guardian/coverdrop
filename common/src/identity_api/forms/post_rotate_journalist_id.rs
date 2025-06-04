use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    form::Form,
    protocol::{
        keys::{
            JournalistIdKeyPair, UnregisteredJournalistIdPublicKey,
            UntrustedUnregisteredJournalistIdPublicKey,
        },
        roles::JournalistId,
    },
};

// This form is provided by the API.
// A journalist signs a form containing an unregistered ID public key and submits
// that to a "queue" of public keys for the identity-api to sign with it's provisioning
// keys.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RotateJournalistIdPublicKeyBody {
    pub new_pk: UntrustedUnregisteredJournalistIdPublicKey,
}

impl RotateJournalistIdPublicKeyBody {
    pub fn new(new_pk: &UnregisteredJournalistIdPublicKey) -> Self {
        let new_pk = new_pk.to_untrusted();

        Self { new_pk }
    }
}

pub type RotateJournalistIdPublicKeyForm = Form<RotateJournalistIdPublicKeyBody, JournalistId>;

impl RotateJournalistIdPublicKeyForm {
    pub fn new(
        new_pk: &UnregisteredJournalistIdPublicKey,
        signing_key_pair: &JournalistIdKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        let body = RotateJournalistIdPublicKeyBody::new(new_pk);
        Self::new_from_form_data(body, signing_key_pair, now)
    }
}
