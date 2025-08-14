use std::path::Path;

use chrono::{DateTime, Utc};

use crate::{
    aws::ssm::{
        client::SsmClient, parameters::ANCHOR_ORG_PK_SSM_PARAMETER, prefix::ParameterPrefix,
    },
    crypto::keys::{
        role::Role,
        serde::StorableKeyMaterial,
        signing::traits::{self, PublicSigningKey},
    },
    protocol::roles::{AnchorOrganization, CoverNodeId, CoverNodeProvisioning, Organization},
    Error,
};

use super::{
    anchor_org_pk, AnchorOrganizationPublicKey, CoverNodeIdKeyPair, CoverNodeMessagingKeyPair,
    CoverNodeProvisioningKeyPair, JournalistProvisioningKeyPair, OrganizationKeyPair,
    UntrustedAnchorOrganizationPublicKey, UntrustedCoverNodeIdKeyPair,
    UntrustedCoverNodeMessagingKeyPair, UntrustedCoverNodeProvisioningKeyPair,
    UntrustedJournalistProvisioningKeyPair, UntrustedOrganizationKeyPair,
};

/// Its very common to load the organization public key from disk since it has to be shipped with the
/// application in order to be trusted.
pub fn load_anchor_org_pks(
    keys_path: impl AsRef<Path>,
    now: DateTime<Utc>,
) -> anyhow::Result<Vec<AnchorOrganizationPublicKey>> {
    let org_pks = UntrustedAnchorOrganizationPublicKey::load_from_directory(&keys_path)?
        .iter()
        .flat_map(|org_pk| {
            anchor_org_pk(org_pk, now).map_err(|e| {
                tracing::warn!(
                    "Failed to trust org_pk {}: {}",
                    hex::encode(org_pk.key.as_bytes()),
                    e,
                );
                e
            })
        })
        .collect::<Vec<AnchorOrganizationPublicKey>>();

    Ok(org_pks)
}

pub async fn load_anchor_org_pks_from_ssm(
    ssm_client: &SsmClient,
    parameter_prefix: &ParameterPrefix,
    now: DateTime<Utc>,
) -> anyhow::Result<Vec<AnchorOrganizationPublicKey>> {
    let parameter = parameter_prefix.get_parameter(ANCHOR_ORG_PK_SSM_PARAMETER);

    let org_pks = ssm_client
        .get_all_parameter_versions(&parameter, 10)
        .await?
        .iter()
        .flat_map(|pk_json| {
            let org_pk = serde_json::from_str(pk_json)?;
            anchor_org_pk(&org_pk, now)
        })
        .collect();

    Ok(org_pks)
}

pub fn load_org_key_pairs(
    keys_path: impl AsRef<Path>,
    now: DateTime<Utc>,
) -> anyhow::Result<Vec<OrganizationKeyPair>> {
    let org_key_pairs = UntrustedOrganizationKeyPair::load_from_directory(&keys_path)?
        .iter()
        .flat_map(|key_pair| key_pair.to_trusted_self_signed(now))
        .collect::<Vec<_>>();

    Ok(org_key_pairs)
}

/// Load latest organization key pair from disk - used for signing new provisioning and system status keys
pub fn load_latest_org_key_pair(
    keys_path: impl AsRef<Path>,
    now: DateTime<Utc>,
) -> anyhow::Result<OrganizationKeyPair> {
    let org_key_pair = UntrustedOrganizationKeyPair::load_from_directory(&keys_path)?
        .iter()
        .flat_map(|key_pair| key_pair.to_trusted_self_signed(now))
        .max_by_key(|key_pair| key_pair.public_key().not_valid_after)
        .ok_or_else(|| Error::LatestKeyPairNotFound(Organization::display()))?;

    Ok(org_key_pair)
}

pub fn load_covernode_provisioning_key_pairs_with_parent<T>(
    keys_path: impl AsRef<Path>,
    anchor_org_pks: &[T],
    now: DateTime<Utc>,
) -> anyhow::Result<Vec<(CoverNodeProvisioningKeyPair, &T)>>
where
    T: traits::PublicSigningKey<AnchorOrganization>,
{
    let covernode_provisioning_key_pairs =
        UntrustedCoverNodeProvisioningKeyPair::load_from_directory(&keys_path)?
            .iter()
            .flat_map(|key_pair| {
                anchor_org_pks.iter().flat_map(|org_pk| {
                    key_pair
                        .to_trusted(org_pk, now)
                        .map(|key_pair| (key_pair, org_pk))
                })
            })
            .collect::<Vec<_>>();

    Ok(covernode_provisioning_key_pairs)
}

pub fn load_covernode_provisioning_key_pairs<T>(
    keys_path: impl AsRef<Path>,
    anchor_org_pks: &[T],
    now: DateTime<Utc>,
) -> anyhow::Result<Vec<CoverNodeProvisioningKeyPair>>
where
    T: traits::PublicSigningKey<AnchorOrganization>,
{
    let covernode_provisioning_key_pairs =
        UntrustedCoverNodeProvisioningKeyPair::load_from_directory(&keys_path)?
            .iter()
            .flat_map(|key_pair| {
                anchor_org_pks
                    .iter()
                    .flat_map(|org_pk| key_pair.to_trusted(org_pk, now))
            })
            .collect::<Vec<_>>();

    Ok(covernode_provisioning_key_pairs)
}

pub fn load_covernode_id_key_pairs(
    keys_path: impl AsRef<Path>,
    covernode_provisioning_pks: &[impl traits::PublicSigningKey<CoverNodeProvisioning>],
    now: DateTime<Utc>,
) -> anyhow::Result<Vec<CoverNodeIdKeyPair>> {
    let covernode_id_key_pairs = UntrustedCoverNodeIdKeyPair::load_from_directory(&keys_path)?
        .iter()
        .flat_map(|key_pair| {
            covernode_provisioning_pks
                .iter()
                .flat_map(|covernode_provisioning_pk| {
                    key_pair.to_trusted(covernode_provisioning_pk, now)
                })
        })
        .inspect(|key_pair| {
            let public_key_hex = hex::encode(&key_pair.public_key().as_bytes()[..8]);
            tracing::debug!("Loaded CoverNode ID key pair: {}", public_key_hex);
        })
        .collect::<Vec<_>>();

    Ok(covernode_id_key_pairs)
}

pub fn load_covernode_msg_key_pairs(
    keys_path: impl AsRef<Path>,
    covernode_id_pks: &[impl traits::PublicSigningKey<CoverNodeId>],
    now: DateTime<Utc>,
) -> anyhow::Result<Vec<CoverNodeMessagingKeyPair>> {
    let covernode_msg_key_pairs =
        UntrustedCoverNodeMessagingKeyPair::load_from_directory(&keys_path)?
            .iter()
            .flat_map(|key_pair| {
                covernode_id_pks
                    .iter()
                    .flat_map(|covernode_id_pk| key_pair.to_trusted(covernode_id_pk, now))
            })
            .collect::<Vec<_>>();

    Ok(covernode_msg_key_pairs)
}

// Journalist

/// Returns journalist provisioning key pairs from the provided keys path and the
/// organization key that verified it.
pub fn load_journalist_provisioning_key_pairs_with_parent<T>(
    keys_path: impl AsRef<Path>,
    anchor_org_pks: &[T],
    now: DateTime<Utc>,
) -> anyhow::Result<Vec<(JournalistProvisioningKeyPair, &T)>>
where
    T: traits::PublicSigningKey<AnchorOrganization>,
{
    let journalist_provisioning_key_pairs =
        UntrustedJournalistProvisioningKeyPair::load_from_directory(&keys_path)?
            .iter()
            .flat_map(|key_pair| {
                anchor_org_pks.iter().flat_map(|org_pk| {
                    key_pair
                        .to_trusted(org_pk, now)
                        .map(|key_pair| (key_pair, org_pk))
                })
            })
            .collect::<Vec<_>>();

    Ok(journalist_provisioning_key_pairs)
}

pub fn load_journalist_provisioning_key_pairs<T>(
    keys_path: impl AsRef<Path>,
    anchor_org_pks: &[T],
    now: DateTime<Utc>,
) -> anyhow::Result<Vec<JournalistProvisioningKeyPair>>
where
    T: traits::PublicSigningKey<AnchorOrganization>,
{
    let journalist_provisioning_key_pairs =
        UntrustedJournalistProvisioningKeyPair::load_from_directory(&keys_path)?
            .iter()
            .flat_map(|key_pair| {
                anchor_org_pks
                    .iter()
                    .flat_map(|org_pk| key_pair.to_trusted(org_pk, now))
            })
            .collect::<Vec<_>>();

    Ok(journalist_provisioning_key_pairs)
}

pub fn load_journalist_provisioning_key_pairs_with_parent_org_pks(
    keys_path: impl AsRef<Path>,
    anchor_org_pks: &[AnchorOrganizationPublicKey],
    now: DateTime<Utc>,
) -> anyhow::Result<Vec<(AnchorOrganizationPublicKey, JournalistProvisioningKeyPair)>> {
    let journalist_provisioning_key_pairs_with_parents =
        UntrustedJournalistProvisioningKeyPair::load_from_directory(&keys_path)?
            .iter()
            .flat_map(|key_pair| {
                anchor_org_pks.iter().flat_map(|org_pk| {
                    let journalist_provisioning_key_pair = key_pair.to_trusted(org_pk, now)?;

                    anyhow::Ok((org_pk.clone(), journalist_provisioning_key_pair))
                })
            })
            .collect::<Vec<_>>();

    Ok(journalist_provisioning_key_pairs_with_parents)
}
