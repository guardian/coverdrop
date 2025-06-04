use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    api::models::journalist_id::JournalistIdentity,
    form::Form,
    protocol::{keys::JournalistProvisioningKeyPair, roles::JournalistProvisioning},
};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PatchJournalistBody {
    pub journalist_id: JournalistIdentity,
    pub display_name: Option<String>,
    pub sort_name: Option<String>,
    pub is_desk: Option<bool>,
    pub description: Option<String>,
}

pub type PatchJournalistForm = Form<PatchJournalistBody, JournalistProvisioning>;

impl PatchJournalistForm {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        journalist_id: JournalistIdentity,
        display_name: Option<String>,
        sort_name: Option<String>,
        is_desk: Option<bool>,
        description: Option<String>,
        signing_key_pair: &JournalistProvisioningKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        let body = PatchJournalistBody {
            journalist_id,
            display_name,
            sort_name,
            is_desk,
            description,
        };

        Self::new_from_form_data(body, signing_key_pair, now)
    }
}
