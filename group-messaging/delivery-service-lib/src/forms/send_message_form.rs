use crate::tls_serialized::TlsSerialized;
use chrono::{DateTime, Utc};
use common::{
    api::models::journalist_id::JournalistIdentity,
    form::Form,
    protocol::{keys::JournalistIdKeyPair, roles::JournalistId},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SendMessageFormBody {
    /// TLS-serialized GroupMessage
    pub message: TlsSerialized,
    pub recipients: Vec<JournalistIdentity>,
}

/// Form for sending an encrypted MLS group message to specified recipients.
/// Used by clients to post messages to the delivery service for distribution to group members.
#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct SendMessageForm(Form<SendMessageFormBody, JournalistId>);

impl SendMessageForm {
    pub fn new(
        message: TlsSerialized,
        recipients: Vec<JournalistIdentity>,
        signing_key_pair: &JournalistIdKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        let body = SendMessageFormBody {
            message,
            recipients,
        };
        let form = Form::new_from_form_data(body, signing_key_pair, now)?;
        Ok(Self(form))
    }
}

impl std::ops::Deref for SendMessageForm {
    type Target = Form<SendMessageFormBody, JournalistId>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
