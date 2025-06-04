use chrono::{DateTime, Utc};

use crate::{
    api::models::messages::journalist_to_covernode_message::EncryptedJournalistToCoverNodeMessage,
    form::Form,
    protocol::{keys::JournalistIdKeyPair, roles::JournalistId},
};

pub type PostJournalistToCoverNodeMessageForm =
    Form<EncryptedJournalistToCoverNodeMessage, JournalistId>;

impl PostJournalistToCoverNodeMessageForm {
    pub fn new(
        j2c_msg: EncryptedJournalistToCoverNodeMessage,
        signing_key_pair: &JournalistIdKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        Self::new_from_form_data(j2c_msg, signing_key_pair, now)
    }
}
