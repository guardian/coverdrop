use chrono::{DateTime, Utc};

use common::{
    form::Form,
    protocol::{keys::JournalistIdKeyPair, roles::JournalistId},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReceiveMessagesFormBody {
    pub ids_greater_than: u32,
}

/// Form for polling new messages from the delivery service.
/// Used by clients to fetch messages with IDs greater than their last successfully-processed message ID.
#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct ReceiveMessagesForm(Form<ReceiveMessagesFormBody, JournalistId>);

impl ReceiveMessagesForm {
    pub fn new(
        ids_greater_than: u32,
        signing_key_pair: &JournalistIdKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        let body = ReceiveMessagesFormBody { ids_greater_than };
        let form = Form::new_from_form_data(body, signing_key_pair, now)?;
        Ok(Self(form))
    }
}

impl std::ops::Deref for ReceiveMessagesForm {
    type Target = Form<ReceiveMessagesFormBody, JournalistId>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
