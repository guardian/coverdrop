use crate::tls_serialized::TlsSerialized;
use crate::MAX_MESSAGE_SIZE_BYTES;
use chrono::{DateTime, Utc};
use common::{
    api::models::sentinel_id::SentinelIdentity,
    form::Form,
    protocol::{keys::SentinelIdKeyPair, roles::SentinelId},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SendMessageFormBody {
    /// TLS-serialized GroupMessage
    pub message: TlsSerialized,
    pub recipients: Vec<SentinelIdentity>,
}

/// Form for sending an encrypted MLS group message to specified recipients.
/// Used by clients to post messages to the delivery service for distribution to group members.
#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct SendMessageForm(Form<SendMessageFormBody, SentinelId>);

impl SendMessageForm {
    pub fn new(
        message: TlsSerialized,
        recipients: Vec<SentinelIdentity>,
        signing_key_pair: &SentinelIdKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        anyhow::ensure!(
            message.len() <= MAX_MESSAGE_SIZE_BYTES,
            "Message size {} bytes exceeds maximum of {} bytes",
            message.len(),
            MAX_MESSAGE_SIZE_BYTES
        );

        let body = SendMessageFormBody {
            message,
            recipients,
        };
        let form = Form::new_from_form_data(body, signing_key_pair, now)?;
        Ok(Self(form))
    }
}

impl std::ops::Deref for SendMessageForm {
    type Target = Form<SendMessageFormBody, SentinelId>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
