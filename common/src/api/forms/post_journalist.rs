use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    api::models::journalist_id::JournalistIdentity,
    client::JournalistStatus,
    form::Form,
    protocol::{keys::JournalistProvisioningKeyPair, roles::JournalistProvisioning},
};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostJournalistBody {
    pub id: JournalistIdentity,
    pub display_name: String,
    pub sort_name: String,
    pub description: String,
    pub is_desk: bool,
    pub status: JournalistStatus,
}

pub type PostJournalistForm = Form<PostJournalistBody, JournalistProvisioning>;

impl PostJournalistForm {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: JournalistIdentity,
        display_name: String,
        sort_name: String,
        description: String,
        is_desk: bool,
        status: JournalistStatus,
        signing_key_pair: &JournalistProvisioningKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        let body = PostJournalistBody {
            id,
            display_name,
            sort_name,
            description,
            is_desk,
            status,
        };

        Self::new_from_form_data(body, signing_key_pair, now)
    }
}
