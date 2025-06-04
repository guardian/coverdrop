use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    form::Form,
    system::{keys::AdminKeyPair, roles::Admin},
};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostLogConfigBody {
    pub rust_log_directive: String,
}

pub type PostLogConfigForm = Form<PostLogConfigBody, Admin>;

impl PostLogConfigForm {
    pub fn new(
        rust_log_directive: String,
        signing_key_pair: &AdminKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        let body = PostLogConfigBody { rust_log_directive };

        Self::new_from_form_data(body, signing_key_pair, now)
    }
}
