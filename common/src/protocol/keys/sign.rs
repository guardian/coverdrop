//! This module contains functions for signing existing, but unregistered keys
//! They are generally used as part of key rotation services.
use chrono::{DateTime, Utc};

use crate::{
    api::models::{
        covernode_id::CoverNodeIdentity, journalist_id::JournalistIdentity,
        sentinel_id::SentinelIdentity,
    },
    crypto::keys::{
        id_key_certificate_data::IdKeyCertificateData, key_certificate_data::KeyCertificateData,
    },
    protocol::constants::{
        COVERNODE_ID_KEY_VALID_DURATION, JOURNALIST_ID_KEY_VALID_DURATION,
        SENTINEL_ID_KEY_VALID_DURATION,
    },
};

use super::{
    generate_child_expiry_not_valid_after, CoverNodeId, CoverNodeIdPublicKey,
    CoverNodeProvisioningKeyPair, JournalistId, JournalistIdPublicKey,
    JournalistProvisioningKeyPair, SentinelId, SentinelIdPublicKey,
    UnregisteredCoverNodeIdPublicKey, UnregisteredJournalistIdPublicKey,
    UnregisteredSentinelIdPublicKey,
};

pub fn sign_covernode_id_pk(
    unsigned_pk: UnregisteredCoverNodeIdPublicKey,
    covernode_provisioning_key_pair: &CoverNodeProvisioningKeyPair,
    now: DateTime<Utc>,
    identity: &CoverNodeIdentity,
) -> CoverNodeIdPublicKey {
    let not_valid_after = generate_child_expiry_not_valid_after(
        COVERNODE_ID_KEY_VALID_DURATION,
        covernode_provisioning_key_pair,
        now,
    );

    // Create legacy certificate
    let certificate_data =
        KeyCertificateData::new_for_signing_key(&unsigned_pk.key, not_valid_after);
    let certificate = covernode_provisioning_key_pair.sign(&certificate_data);

    // Create new signature, signing over the cover node's identity
    let id_cert_data =
        IdKeyCertificateData::new::<CoverNodeId>(&unsigned_pk.key, not_valid_after, identity);
    let signature = covernode_provisioning_key_pair.sign(&id_cert_data);

    CoverNodeIdPublicKey::new(
        unsigned_pk.key,
        Some(signature),
        certificate,
        not_valid_after,
    )
}

pub fn sign_journalist_id_pk(
    unsigned_pk: UnregisteredJournalistIdPublicKey,
    journalist_provisioning_key_pair: &JournalistProvisioningKeyPair,
    now: DateTime<Utc>,
    identity: &JournalistIdentity,
) -> JournalistIdPublicKey {
    let not_valid_after = generate_child_expiry_not_valid_after(
        JOURNALIST_ID_KEY_VALID_DURATION,
        journalist_provisioning_key_pair,
        now,
    );

    // Create legacy certificate
    let certificate_data =
        KeyCertificateData::new_for_signing_key(&unsigned_pk.key, not_valid_after);
    let certificate = journalist_provisioning_key_pair.sign(&certificate_data);

    // Create new signature, signing over the journalist's identity
    let id_cert_data =
        IdKeyCertificateData::new::<JournalistId>(&unsigned_pk.key, not_valid_after, identity);
    let signature = journalist_provisioning_key_pair.sign(&id_cert_data);

    JournalistIdPublicKey::new(
        unsigned_pk.key,
        Some(signature),
        certificate,
        not_valid_after,
    )
}

pub fn sign_sentinel_id_pk(
    unsigned_pk: UnregisteredSentinelIdPublicKey,
    journalist_provisioning_key_pair: &JournalistProvisioningKeyPair,
    now: DateTime<Utc>,
    identity: &SentinelIdentity,
) -> SentinelIdPublicKey {
    let not_valid_after = generate_child_expiry_not_valid_after(
        SENTINEL_ID_KEY_VALID_DURATION,
        journalist_provisioning_key_pair,
        now,
    );

    // Create legacy certificate
    let certificate_data =
        KeyCertificateData::new_for_signing_key(&unsigned_pk.key, not_valid_after);
    let certificate = journalist_provisioning_key_pair.sign(&certificate_data);

    // Create new signature, signing over the sentinel user's identity
    let id_cert_data =
        IdKeyCertificateData::new::<SentinelId>(&unsigned_pk.key, not_valid_after, identity);
    let signature = journalist_provisioning_key_pair.sign(&id_cert_data);

    SentinelIdPublicKey::new(
        unsigned_pk.key,
        Some(signature),
        certificate,
        not_valid_after,
    )
}
