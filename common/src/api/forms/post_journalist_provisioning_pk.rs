use chrono::{DateTime, Utc};

use crate::{
    form::Form,
    protocol::{
        keys::{OrganizationKeyPair, UntrustedJournalistProvisioningPublicKey},
        roles::Organization,
    },
};

pub type PostJournalistProvisioningPublicKeyForm =
    Form<UntrustedJournalistProvisioningPublicKey, Organization>;

impl PostJournalistProvisioningPublicKeyForm {
    pub fn new(
        journalist_provisioning_pk: UntrustedJournalistProvisioningPublicKey,
        signing_key_pair: &OrganizationKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        Self::new_from_form_data(journalist_provisioning_pk, signing_key_pair, now)
    }
}
