use chrono::{DateTime, Utc};

use crate::{
    form::Form,
    protocol::{
        keys::{OrganizationKeyPair, UntrustedCoverNodeProvisioningPublicKey},
        roles::Organization,
    },
};

pub type PostCoverNodeProvisioningPublicKeyForm =
    Form<UntrustedCoverNodeProvisioningPublicKey, Organization>;

impl PostCoverNodeProvisioningPublicKeyForm {
    pub fn new(
        covernode_provisioning_pk: UntrustedCoverNodeProvisioningPublicKey,
        signing_key_pair: &OrganizationKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        Self::new_from_form_data(covernode_provisioning_pk, signing_key_pair, now)
    }
}
