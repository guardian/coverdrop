use chrono::{DateTime, Utc};

use crate::{
    form::Form,
    protocol::{
        keys::{JournalistIdKeyPair, UntrustedJournalistMessagingPublicKey},
        roles::JournalistId,
    },
};

pub type PostJournalistMessagingPublicKeyForm =
    Form<UntrustedJournalistMessagingPublicKey, JournalistId>;

impl PostJournalistMessagingPublicKeyForm {
    pub fn new(
        journalist_msg_pk: UntrustedJournalistMessagingPublicKey,
        signing_key_pair: &JournalistIdKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        Self::new_from_form_data(journalist_msg_pk, signing_key_pair, now)
    }
}
