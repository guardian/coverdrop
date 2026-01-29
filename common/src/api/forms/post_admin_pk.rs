use chrono::{DateTime, Utc};

use crate::{
    form::{Form, BUNDLE_FORM_TTL},
    protocol::{keys::OrganizationKeyPair, roles::Organization},
    system::keys::UntrustedAdminPublicKey,
};

pub type PostAdminPublicKeyForm = Form<UntrustedAdminPublicKey, Organization>;

impl PostAdminPublicKeyForm {
    pub fn new(
        admin_pk: UntrustedAdminPublicKey,
        signing_key_pair: &OrganizationKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        Self::new_from_form_data(admin_pk, signing_key_pair, now)
    }

    pub fn new_for_bundle(
        admin_pk: UntrustedAdminPublicKey,
        signing_key_pair: &OrganizationKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        Self::new_from_form_data_custom_ttl(admin_pk, signing_key_pair, BUNDLE_FORM_TTL, now)
    }
}
