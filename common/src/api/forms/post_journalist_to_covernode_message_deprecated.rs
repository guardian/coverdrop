use chrono::{DateTime, Utc};

use crate::{
    api::models::messages::journalist_to_covernode_message::EncryptedJournalistToCoverNodeMessage,
    form::Form,
    protocol::{keys::JournalistIdKeyPair, roles::JournalistId},
};

// TODO remove https://github.com/guardian/coverdrop-internal/issues/4087
#[deprecated]
pub type PostJournalistToCoverNodeMessageFormDeprecated =
    Form<EncryptedJournalistToCoverNodeMessage, JournalistId>;

#[allow(deprecated)]
impl PostJournalistToCoverNodeMessageFormDeprecated {
    pub fn new(
        j2c_msg: EncryptedJournalistToCoverNodeMessage,
        signing_key_pair: &JournalistIdKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        Self::new_from_form_data(j2c_msg, signing_key_pair, now)
    }
}
