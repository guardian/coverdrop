use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    form::Form,
    protocol::{
        keys::{
            CoverNodeIdKeyPair, UnregisteredCoverNodeIdPublicKey,
            UntrustedUnregisteredCoverNodeIdPublicKey,
        },
        roles::CoverNodeId,
    },
};

/// Body for the POST /covernodes/me/rotate-id-key endpoint
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RotateCoverNodeIdPublicKeyBody {
    pub new_pk: UntrustedUnregisteredCoverNodeIdPublicKey,
}

impl RotateCoverNodeIdPublicKeyBody {
    pub fn new(new_pk: &UnregisteredCoverNodeIdPublicKey) -> Self {
        let new_pk = new_pk.to_untrusted();

        Self { new_pk }
    }
}

pub type RotateCoverNodeIdPublicKeyForm = Form<RotateCoverNodeIdPublicKeyBody, CoverNodeId>;

impl RotateCoverNodeIdPublicKeyForm {
    pub fn new(
        new_pk: &UnregisteredCoverNodeIdPublicKey,
        signing_key_pair: &CoverNodeIdKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        let body = RotateCoverNodeIdPublicKeyBody::new(new_pk);
        Self::new_from_form_data(body, signing_key_pair, now)
    }
}
