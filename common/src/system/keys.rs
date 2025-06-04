use std::path::Path;

use chrono::{DateTime, Duration, Utc};

use crate::{
    crypto::keys::{
        serde::StorableKeyMaterial,
        signing::{traits, SignedPublicSigningKey, SignedSigningKeyPair, UnsignedSigningKeyPair},
        untrusted::signing::{UntrustedSignedPublicSigningKey, UntrustedSignedSigningKeyPair},
    },
    protocol::{
        keys::{OrganizationKeyPair, OrganizationPublicKey},
        roles::AnchorOrganization,
    },
};

use super::{constants::ADMIN_KEY_VALID_DURATION_SECONDS, roles::Admin};

pub type UntrustedAdminPublicKey = UntrustedSignedPublicSigningKey<Admin>;
pub type UntrustedAdminKeyPair = UntrustedSignedSigningKeyPair<Admin>;
pub type AdminPublicKey = SignedPublicSigningKey<Admin>;
pub type AdminKeyPair = SignedSigningKeyPair<Admin>;

pub fn generate_admin_key_pair(
    org_key_pair: &OrganizationKeyPair,
    now: DateTime<Utc>,
) -> AdminKeyPair {
    let not_valid_after = now + Duration::seconds(ADMIN_KEY_VALID_DURATION_SECONDS);

    UnsignedSigningKeyPair::generate().to_signed_key_pair(org_key_pair, not_valid_after)
}

pub fn verify_admin_pk(
    untrusted: &UntrustedAdminPublicKey,
    org_pk: &OrganizationPublicKey,
    now: DateTime<Utc>,
) -> anyhow::Result<AdminPublicKey> {
    untrusted.to_trusted(org_pk, now)
}

pub fn load_admin_key_pair(
    keys_path: impl AsRef<Path>,
    org_pks: &[impl traits::PublicSigningKey<AnchorOrganization>],
    now: DateTime<Utc>,
) -> anyhow::Result<Vec<AdminKeyPair>> {
    let key_pair = UntrustedAdminKeyPair::load_from_directory(&keys_path)?
        .iter()
        .flat_map(|key_pair| {
            org_pks
                .iter()
                .flat_map(|org_pk| key_pair.to_trusted(org_pk, now))
        })
        .collect::<Vec<_>>();

    Ok(key_pair)
}
