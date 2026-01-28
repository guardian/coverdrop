//! Various functions for adding type safety to the key conversion functions
use chrono::{DateTime, Utc};

use crate::crypto::keys::{signing::PublicSigningKey, untrusted::UntrustedKeyError};

use super::*;

/// Trust a serialized trusted organization public key, this is used when reading a key from a trusted store
/// such as the local file system or a journalist vault.
///
/// We still perform the self-signing certificate check in case the key has expired
pub fn anchor_org_pk(
    untrusted: &UntrustedAnchorOrganizationPublicKey,
    now: DateTime<Utc>,
) -> anyhow::Result<AnchorOrganizationPublicKey> {
    let self_verification_pk = PublicSigningKey::<AnchorOrganization>::new(untrusted.key);

    untrusted.to_trusted(&self_verification_pk, now)
}

pub fn verify_organization_pk(
    untrusted: &UntrustedOrganizationPublicKey,
    anchor_org_pk: &AnchorOrganizationPublicKey,
    now: DateTime<Utc>,
) -> anyhow::Result<OrganizationPublicKey> {
    if untrusted.key == anchor_org_pk.key
        && untrusted.certificate == anchor_org_pk.certificate
        && untrusted.not_valid_after == anchor_org_pk.not_valid_after
    {
        let self_verification_pk = PublicSigningKey::<Organization>::new(untrusted.key);

        untrusted.to_trusted(&self_verification_pk, now)
    } else {
        anyhow::bail!(
            "Trusted organization public key does not match untrusted organization public key"
        )
    }
}

pub fn verify_covernode_provisioning_pk(
    untrusted: &UntrustedCoverNodeProvisioningPublicKey,
    org_pk: &OrganizationPublicKey,
    now: DateTime<Utc>,
) -> anyhow::Result<CoverNodeProvisioningPublicKey> {
    untrusted.to_trusted(org_pk, now)
}

pub fn verify_covernode_id_pk(
    untrusted: &UntrustedCoverNodeIdPublicKey,
    covernode_provisioning_pk: &CoverNodeProvisioningPublicKey,
    now: DateTime<Utc>,
) -> anyhow::Result<CoverNodeIdPublicKey> {
    untrusted.to_trusted(covernode_provisioning_pk, now)
}

pub fn verify_covernode_messaging_pk(
    untrusted: &UntrustedCoverNodeMessagingPublicKey,
    covernode_id_pk: &CoverNodeIdPublicKey,
    now: DateTime<Utc>,
) -> anyhow::Result<CoverNodeMessagingPublicKey> {
    Ok(untrusted.to_trusted(covernode_id_pk, now)?)
}

pub fn verify_journalist_provisioning_pk(
    untrusted: &UntrustedJournalistProvisioningPublicKey,
    org_pk: &OrganizationPublicKey,
    now: DateTime<Utc>,
) -> anyhow::Result<JournalistProvisioningPublicKey> {
    untrusted.to_trusted(org_pk, now)
}

pub fn verify_journalist_id_pk(
    untrusted: &UntrustedJournalistIdPublicKey,
    journalist_provisioning_pk: &JournalistProvisioningPublicKey,
    now: DateTime<Utc>,
) -> anyhow::Result<JournalistIdPublicKey> {
    untrusted.to_trusted(journalist_provisioning_pk, now)
}

pub fn verify_journalist_messaging_pk(
    untrusted: &UntrustedJournalistMessagingPublicKey,
    journalist_id_pk: &JournalistIdPublicKey,
    now: DateTime<Utc>,
) -> Result<JournalistMessagingPublicKey, UntrustedKeyError> {
    untrusted.to_trusted(journalist_id_pk, now)
}
