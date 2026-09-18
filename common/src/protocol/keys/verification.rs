//! Various functions for adding type safety to the key conversion functions
use chrono::{DateTime, Utc};

use crate::{
    api::models::{
        covernode_id::CoverNodeIdentity, journalist_id::JournalistIdentity,
        sentinel_id::SentinelIdentity,
    },
    crypto::keys::{signing::PublicSigningKey, untrusted::UntrustedKeyError},
};

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
    identity: &CoverNodeIdentity,
) -> anyhow::Result<CoverNodeIdPublicKey> {
    untrusted.to_trusted_with_identity(covernode_provisioning_pk, now, identity)
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
    identity: &JournalistIdentity,
) -> anyhow::Result<JournalistIdPublicKey> {
    untrusted.to_trusted_with_identity(journalist_provisioning_pk, now, identity)
}

pub fn verify_journalist_messaging_pk(
    untrusted: &UntrustedJournalistMessagingPublicKey,
    journalist_id_pk: &JournalistIdPublicKey,
    now: DateTime<Utc>,
) -> Result<JournalistMessagingPublicKey, UntrustedKeyError> {
    untrusted.to_trusted(journalist_id_pk, now)
}

pub fn verify_sentinel_id_pk(
    untrusted: &UntrustedSentinelIdPublicKey,
    journalist_provisioning_pk: &JournalistProvisioningPublicKey,
    now: DateTime<Utc>,
    identity: &SentinelIdentity,
) -> anyhow::Result<SentinelIdPublicKey> {
    untrusted.to_trusted_with_identity(journalist_provisioning_pk, now, identity)
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::protocol::keys::{
        generation::test::generate_protocol_keys, OrganizationPublicKeyFamilyList,
        UntrustedOrganizationPublicKeyFamilyList,
    };

    use super::*;

    use std::collections::HashMap;

    use crate::{
        api::models::journalist_id::JournalistIdentity,
        protocol::keys::{
            generation::{
                generate_journalist_id_key_pair, generate_journalist_messaging_key_pair,
                generate_journalist_provisioning_key_pair, generate_organization_key_pair,
            },
            CoverDropPublicKeyHierarchy, CoverNodeProvisioningPublicKeyFamilyList,
            JournalistIdPublicKeyFamily, JournalistIdPublicKeyFamilyList,
            JournalistProvisioningPublicKeyFamily, JournalistProvisioningPublicKeyFamilyList,
            OrganizationPublicKeyFamily,
        },
    };

    /// Strips all `"signature"` fields from a JSON value, simulating pre-migration wire data.
    fn strip_all_signatures(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::Object(map) => {
                map.remove("signature");
                for v in map.values_mut() {
                    strip_all_signatures(v);
                }
            }
            serde_json::Value::Array(arr) => {
                for v in arr.iter_mut() {
                    strip_all_signatures(v);
                }
            }
            _ => {}
        }
    }

    /// Asserts every key object (identified by having a "certificate") also has a "signature".
    fn assert_all_signatures_present(value: &serde_json::Value) {
        match value {
            serde_json::Value::Object(map) => {
                if map.contains_key("certificate") {
                    assert!(
                        map.contains_key("signature"),
                        "key missing signature: {map:?}"
                    );
                }
                for v in map.values() {
                    assert_all_signatures_present(v);
                }
            }
            serde_json::Value::Array(arr) => {
                for v in arr {
                    assert_all_signatures_present(v);
                }
            }
            _ => {}
        }
    }

    /// Randomly strips `"signature"` fields from a JSON value, simulating mid-migration data.
    fn randomly_strip_signatures(value: &mut serde_json::Value, rng: &mut impl rand::Rng) {
        match value {
            serde_json::Value::Object(map) => {
                if map.contains_key("signature") && rng.gen_bool(0.5) {
                    map.remove("signature");
                }
                for v in map.values_mut() {
                    randomly_strip_signatures(v, rng);
                }
            }
            serde_json::Value::Array(arr) => {
                for v in arr.iter_mut() {
                    randomly_strip_signatures(v, rng);
                }
            }
            _ => {}
        }
    }

    #[test]
    fn verify_hierarchy_pre_migration_all_signatures_none() {
        let now = Utc::now();
        let keys = generate_protocol_keys(now);

        let anchor = anchor_org_pk(&keys.org_pk.to_untrusted().to_tofu_anchor(), now)
            .expect("anchor org pk");

        // Serialize hierarchy, strip all signature fields to simulate pre-migration data
        let mut json = serde_json::to_value(keys.hierarchy.to_untrusted()).unwrap();
        strip_all_signatures(&mut json);
        println!("Untrusted hierarchy without signatures: {:#?}", json);

        let untrusted: UntrustedOrganizationPublicKeyFamilyList =
            serde_json::from_value(json).unwrap();

        let hierarchy = OrganizationPublicKeyFamilyList::from_untrusted(untrusted, &[anchor], now);

        assert!(hierarchy.latest_org_pk().is_some());
        assert!(hierarchy.latest_covernode_provisioning_pk().is_some());
        assert!(hierarchy.latest_covernode_id_pk_iter().next().is_some());
        assert!(hierarchy.latest_covernode_msg_pk_iter().next().is_some());
        assert!(hierarchy.latest_journalist_provisioning_pk().is_some());
        assert!(hierarchy.latest_journalist_id_pk_iter().next().is_some());
        assert!(hierarchy.latest_journalist_msg_pk_iter().next().is_some());
        assert!(hierarchy.latest_sentinel_id_pk_iter().next().is_some());
    }

    #[test]
    fn verify_hierarchy_mid_migration_random_signatures_some() {
        use rand::SeedableRng;

        let now = Utc::now();
        let keys = generate_protocol_keys(now);

        let anchor = anchor_org_pk(&keys.org_pk.to_untrusted().to_tofu_anchor(), now)
            .expect("anchor org pk");

        // Run multiple iterations with different seeds to cover various combinations
        for seed in 0..20u64 {
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            let mut json = serde_json::to_value(keys.hierarchy.to_untrusted()).unwrap();
            randomly_strip_signatures(&mut json, &mut rng);

            let untrusted: UntrustedOrganizationPublicKeyFamilyList =
                serde_json::from_value(json).unwrap();

            let hierarchy =
                OrganizationPublicKeyFamilyList::from_untrusted(untrusted, &[anchor.clone()], now);

            assert!(
                hierarchy.latest_org_pk().is_some(),
                "seed {seed}: org pk missing"
            );
            assert!(
                hierarchy.latest_covernode_provisioning_pk().is_some(),
                "seed {seed}: covernode provisioning pk missing"
            );
            assert!(
                hierarchy.latest_covernode_id_pk_iter().next().is_some(),
                "seed {seed}: covernode id pk missing"
            );
            assert!(
                hierarchy.latest_covernode_msg_pk_iter().next().is_some(),
                "seed {seed}: covernode msg pk missing"
            );
            assert!(
                hierarchy.latest_journalist_provisioning_pk().is_some(),
                "seed {seed}: journalist provisioning pk missing"
            );
            assert!(
                hierarchy.latest_journalist_id_pk_iter().next().is_some(),
                "seed {seed}: journalist id pk missing"
            );
            assert!(
                hierarchy.latest_journalist_msg_pk_iter().next().is_some(),
                "seed {seed}: journalist msg pk missing"
            );
            assert!(
                hierarchy.latest_sentinel_id_pk_iter().next().is_some(),
                "seed {seed}: sentinel id pk missing"
            );
        }
    }

    #[test]
    fn verify_hierarchy_post_migration_all_signatures_present() {
        let now = Utc::now();
        let keys = generate_protocol_keys(now);

        let anchor = anchor_org_pk(&keys.org_pk.to_untrusted().to_tofu_anchor(), now)
            .expect("anchor org pk");

        // Post-migration: all keys have signature fields, use hierarchy as-is
        let untrusted = keys.hierarchy.to_untrusted();

        // Assert every key object that has a "certificate" also has a "signature"
        let json = serde_json::to_value(&untrusted).unwrap();
        assert_all_signatures_present(&json);

        let hierarchy = OrganizationPublicKeyFamilyList::from_untrusted(untrusted, &[anchor], now);

        assert!(hierarchy.latest_org_pk().is_some());
        assert!(hierarchy.latest_covernode_provisioning_pk().is_some());
        assert!(hierarchy.latest_covernode_id_pk_iter().next().is_some());
        assert!(hierarchy.latest_covernode_msg_pk_iter().next().is_some());
        assert!(hierarchy.latest_journalist_provisioning_pk().is_some());
        assert!(hierarchy.latest_journalist_id_pk_iter().next().is_some());
        assert!(hierarchy.latest_journalist_msg_pk_iter().next().is_some());
        assert!(hierarchy.latest_sentinel_id_pk_iter().next().is_some());
    }

    /// Asserts that if a journalist's identity keys are swapped, the verification fails for the swapped key.
    #[test]
    fn verification_fails_when_journalist_identity_keys_are_swapped() {
        let now = Utc::now();

        let org_key_pair = generate_organization_key_pair(now);
        let journalist_provisioning_key_pair =
            generate_journalist_provisioning_key_pair(&org_key_pair, now);

        let journalist_a_id = JournalistIdentity::new("journalist_a").unwrap();
        let journalist_b_id = JournalistIdentity::new("journalist_b").unwrap();

        // Generate legitimate keys for both journalists
        let journalist_a_id_key_pair = generate_journalist_id_key_pair(
            &journalist_provisioning_key_pair,
            now,
            &journalist_a_id,
        );
        let journalist_a_msg_key_pair =
            generate_journalist_messaging_key_pair(&journalist_a_id_key_pair, now);

        // Build hierarchy where journalist B's entry contains journalist A's keys
        let hierarchy = CoverDropPublicKeyHierarchy::new(vec![OrganizationPublicKeyFamily::new(
            org_key_pair.public_key().clone(),
            CoverNodeProvisioningPublicKeyFamilyList::new(vec![]),
            JournalistProvisioningPublicKeyFamilyList::new(vec![
                JournalistProvisioningPublicKeyFamily::new(
                    journalist_provisioning_key_pair.public_key().clone(),
                    HashMap::from([
                        (
                            // Attacker leaves journalist A's key untouched
                            journalist_a_id.clone(),
                            JournalistIdPublicKeyFamilyList::new(vec![
                                JournalistIdPublicKeyFamily::new(
                                    journalist_a_id_key_pair.public_key().clone(),
                                    vec![journalist_a_msg_key_pair.public_key().clone()],
                                ),
                            ]),
                        ),
                        (
                            // Attacker places journalist A's keys under journalist B's identity
                            journalist_b_id.clone(),
                            JournalistIdPublicKeyFamilyList::new(vec![
                                JournalistIdPublicKeyFamily::new(
                                    journalist_a_id_key_pair.public_key().clone(),
                                    vec![journalist_a_msg_key_pair.public_key().clone()],
                                ),
                            ]),
                        ),
                    ]),
                    HashMap::new(),
                ),
            ]),
            None,
        )]);

        let anchor = anchor_org_pk(
            &org_key_pair.public_key().to_untrusted().to_tofu_anchor(),
            now,
        )
        .expect("anchor org pk");

        let untrusted = hierarchy.to_untrusted();
        let verified = OrganizationPublicKeyFamilyList::from_untrusted(untrusted, &[anchor], now);

        // Journalist A's id key should have verified keys since they were signed for "journalist_a"
        assert_eq!(
            verified.latest_journalist_id_pk(&journalist_a_id),
            Some(&journalist_a_id_key_pair.public_key().clone()),
        );
        // Journalist B's id key should have no verified keys since A's key was signed for "journalist_a"
        assert!(
            verified.latest_journalist_id_pk(&journalist_b_id).is_none(),
            "swapped journalist key should fail identity verification"
        );
    }
}
