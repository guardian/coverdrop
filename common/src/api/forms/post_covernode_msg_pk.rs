use chrono::{DateTime, Utc};

use crate::{
    form::Form,
    protocol::{
        keys::{CoverNodeIdKeyPair, UntrustedCoverNodeMessagingPublicKey},
        roles::CoverNodeId,
    },
};
pub type PostCoverNodeMessagingPublicKeyForm =
    Form<UntrustedCoverNodeMessagingPublicKey, CoverNodeId>;

impl PostCoverNodeMessagingPublicKeyForm {
    pub fn new(
        covernode_msg_pk: UntrustedCoverNodeMessagingPublicKey,
        signing_key_pair: &CoverNodeIdKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        Self::new_from_form_data(covernode_msg_pk, signing_key_pair, now)
    }
}
