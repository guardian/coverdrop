//! This module contains functions for signing existing, but unregistered keys
//! They are generally used as part of key rotation services.
use chrono::{DateTime, Duration, Utc};

use crate::{
    crypto::keys::key_certificate_data::KeyCertificateData,
    protocol::constants::{
        COVERNODE_ID_KEY_VALID_DURATION_SECONDS, JOURNALIST_ID_KEY_VALID_DURATION_SECONDS,
    },
};

use super::{
    generate_child_expiry_not_valid_after, CoverNodeIdPublicKey, CoverNodeProvisioningKeyPair,
    JournalistIdPublicKey, JournalistProvisioningKeyPair, UnregisteredCoverNodeIdPublicKey,
    UnregisteredJournalistIdPublicKey,
};

pub fn sign_covernode_id_pk(
    unsigned_pk: UnregisteredCoverNodeIdPublicKey,
    covernode_provisioning_key_pair: &CoverNodeProvisioningKeyPair,
    now: DateTime<Utc>,
) -> CoverNodeIdPublicKey {
    let not_valid_after = generate_child_expiry_not_valid_after(
        Duration::seconds(COVERNODE_ID_KEY_VALID_DURATION_SECONDS),
        covernode_provisioning_key_pair,
        now,
    );

    let certificate_data =
        KeyCertificateData::new_for_signing_key(&unsigned_pk.key, not_valid_after);
    let certificate = covernode_provisioning_key_pair.sign(&certificate_data);

    CoverNodeIdPublicKey::new(unsigned_pk.key, certificate, not_valid_after)
}

pub fn sign_journalist_id_pk(
    unsigned_pk: UnregisteredJournalistIdPublicKey,
    journalist_provisioning_key_pair: &JournalistProvisioningKeyPair,
    now: DateTime<Utc>,
) -> JournalistIdPublicKey {
    let not_valid_after = generate_child_expiry_not_valid_after(
        Duration::seconds(JOURNALIST_ID_KEY_VALID_DURATION_SECONDS),
        journalist_provisioning_key_pair,
        now,
    );

    let certificate_data =
        KeyCertificateData::new_for_signing_key(&unsigned_pk.key, not_valid_after);
    let certificate = journalist_provisioning_key_pair.sign(&certificate_data);

    JournalistIdPublicKey::new(unsigned_pk.key, certificate, not_valid_after)
}
