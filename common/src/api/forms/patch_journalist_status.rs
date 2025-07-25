use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    api::models::journalist_id::JournalistIdentity,
    client::JournalistStatus,
    form::Form,
    protocol::{keys::JournalistIdKeyPair, roles::JournalistId},
};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PatchJournalistStatusBody {
    pub journalist_id: JournalistIdentity,
    pub status: JournalistStatus,
}

pub type PatchJournalistStatusForm = Form<PatchJournalistStatusBody, JournalistId>;

impl PatchJournalistStatusForm {
    pub fn new(
        journalist_id: JournalistIdentity,
        status: JournalistStatus,
        signing_key_pair: &JournalistIdKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        let body = PatchJournalistStatusBody {
            journalist_id,
            status,
        };

        Self::new_from_form_data(body, signing_key_pair, now)
    }
}
