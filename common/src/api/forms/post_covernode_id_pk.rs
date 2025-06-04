use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    api::models::covernode_id::CoverNodeIdentity,
    form::Form,
    protocol::{
        keys::{CoverNodeProvisioningKeyPair, UntrustedCoverNodeIdPublicKey},
        roles::CoverNodeProvisioning,
    },
};

#[derive(Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct PostCoverNodeIdPublicKeyBody {
    pub covernode_id: CoverNodeIdentity,
    pub covernode_id_pk: UntrustedCoverNodeIdPublicKey,
}

pub type PostCoverNodeIdPublicKeyForm = Form<PostCoverNodeIdPublicKeyBody, CoverNodeProvisioning>;

pub type PostCoverNodeIdPublicKeyFormForBundle =
    Form<PostCoverNodeIdPublicKeyBody, CoverNodeProvisioning>;

impl PostCoverNodeIdPublicKeyForm {
    pub fn new(
        covernode_id: CoverNodeIdentity,
        covernode_id_pk: UntrustedCoverNodeIdPublicKey,
        signing_key_pair: &CoverNodeProvisioningKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        let body = PostCoverNodeIdPublicKeyBody {
            covernode_id,
            covernode_id_pk,
        };
        Self::new_from_form_data(body, signing_key_pair, now)
    }

    pub fn new_for_bundle(
        covernode_id: CoverNodeIdentity,
        covernode_id_pk: UntrustedCoverNodeIdPublicKey,
        signing_key_pair: &CoverNodeProvisioningKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        let body = PostCoverNodeIdPublicKeyBody {
            covernode_id,
            covernode_id_pk,
        };
        Self::new_from_form_data_custom_ttl(body, signing_key_pair, chrono::Duration::days(1), now)
    }
}
