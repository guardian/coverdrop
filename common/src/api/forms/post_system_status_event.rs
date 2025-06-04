use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    api::models::general::StatusEvent,
    form::Form,
    system::{keys::AdminKeyPair, roles::Admin},
};

#[derive(Serialize, Deserialize)]
pub struct PostSystemStatusEventBody {
    pub status: StatusEvent,
}

impl PostSystemStatusEventBody {
    pub fn new(status: StatusEvent) -> Self {
        Self { status }
    }
}

pub type PostSystemStatusEventForm = Form<PostSystemStatusEventBody, Admin>;

impl PostSystemStatusEventForm {
    pub fn new(
        status: StatusEvent,
        signing_key_pair: &AdminKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        let body = PostSystemStatusEventBody::new(status);
        Self::new_from_form_data(body, signing_key_pair, now)
    }
}
