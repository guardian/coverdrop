use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    api::models::journalist_id::JournalistIdentity,
    form::Form,
    protocol::{
        keys::{JournalistProvisioningKeyPair, UntrustedJournalistIdPublicKey},
        roles::JournalistProvisioning,
    },
};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostJournalistIdPublicKeyBody {
    pub journalist_id: JournalistIdentity,
    pub journalist_id_pk: UntrustedJournalistIdPublicKey,
    /// This form is signed by the provisioning key and we trust
    ///  the identity API to not lie about this.
    pub from_queue: bool,
}

pub type PostJournalistIdPublicKeyForm =
    Form<PostJournalistIdPublicKeyBody, JournalistProvisioning>;

impl PostJournalistIdPublicKeyForm {
    pub fn new(
        journalist_id: JournalistIdentity,
        journalist_id_pk: UntrustedJournalistIdPublicKey,
        from_queue: bool,
        signing_key_pair: &JournalistProvisioningKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        let body = PostJournalistIdPublicKeyBody {
            journalist_id,
            journalist_id_pk,
            from_queue,
        };

        Self::new_from_form_data(body, signing_key_pair, now)
    }
}
