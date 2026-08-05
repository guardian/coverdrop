use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    api::models::{
        message_id::MessageId,
        messages::journalist_to_covernode_message::EncryptedJournalistToCoverNodeMessage,
    },
    form::Form,
    protocol::{keys::JournalistIdKeyPair, roles::JournalistId},
};

#[derive(Serialize, Deserialize, Clone)]
pub struct PostJournalistToCoverNodeMessageBody {
    pub message: EncryptedJournalistToCoverNodeMessage,
    pub deduplication_id: MessageId,
}

pub type PostJournalistToCoverNodeMessageForm =
    Form<PostJournalistToCoverNodeMessageBody, JournalistId>;

impl PostJournalistToCoverNodeMessageForm {
    pub fn new(
        j2c_msg: EncryptedJournalistToCoverNodeMessage,
        deduplication_id: MessageId,
        signing_key_pair: &JournalistIdKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        let body = PostJournalistToCoverNodeMessageBody {
            message: j2c_msg,
            deduplication_id,
        };
        Self::new_from_form_data(body, signing_key_pair, now)
    }
}
