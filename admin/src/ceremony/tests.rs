#[cfg(test)]
mod test {
    use crate::ceremony::admin_key_pair_bundle::AdminKeyPairBundle;
    use crate::ceremony::anchor_public_key_bundle::AnchorOrganizationPublicKeyBundle;
    use crate::ceremony::organization_key_pair_bundle::OrganizationKeyPairsBundle;
    use crate::ceremony::provisioning_key_pairs_bundle::{
        CoverNodeProvisioningKeyPairBundle, JournalistProvisioningKeyPairBundle,
    };
    use crate::ceremony::public_key_forms_bundle::PublicKeyFormsBundle;
    use crate::ceremony::set_system_status_available_bundle::SetSystemStatusAvailableBundle;
    use crate::ceremony::state_machine::CeremonyState;
    use crate::ceremony::CeremonyStep;
    use crate::generate_organization_key_pair;
    use common::api::models::covernode_id::CoverNodeIdentity;
    use common::protocol::keys::{load_org_key_pairs, UntrustedCoverNodeIdKeyPair};
    use common::time;
    use covernode_database::Database;
    use serde::de::DeserializeOwned;
    use std::collections::{HashMap, HashSet};
    use std::path::Path;
    use std::{fs::File, num::NonZeroU8};
    use tempfile::tempdir_in;

    /// Utility function to check that the paths in the ceremony state
    /// maps to a file on disk that can be deserialized to the correct Rust struct `T`
    fn exists_on_disk_and_can_be_deserialized<T>(path: impl AsRef<Path>) -> bool
    where
        T: DeserializeOwned,
    {
        let reader = File::open(path).expect("Open file");
        serde_json::from_reader::<_, T>(reader).is_ok()
    }

    #[tokio::test]
    async fn each_step_produces_expected_bundle_and_bundles_from_previous_steps_are_preserved() {
        let temp_dir = tempdir_in(std::env::current_dir().unwrap()).unwrap();
        let covernode_count = NonZeroU8::new(1).unwrap();

        let mut ceremony = CeremonyStep::new();
        let mut state = CeremonyState::new(
            &temp_dir,
            covernode_count,
            true,
            "some-password".to_string(),
            None,
            time::now(),
        );

        while let Some(step) = ceremony.next() {
            ceremony = step;
            ceremony.execute(&mut state).await.expect("Execute step");
            let state = state.clone();
            match ceremony {
                CeremonyStep::Zero => {
                    assert!(state.org_key_pair_bundle.is_none());
                }
                CeremonyStep::One => {
                    // Assert all bundles are there and can be deserialized
                    assert!(state.org_key_pair_bundle.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<OrganizationKeyPairsBundle>
                    ));
                }
                CeremonyStep::Two => {
                    assert!(state.org_key_pair_bundle.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<OrganizationKeyPairsBundle>
                    ));
                    assert!(
                        state.journalist_provisioning_key_pair_bundle.is_some_and(
                            exists_on_disk_and_can_be_deserialized::<
                                JournalistProvisioningKeyPairBundle,
                            >
                        )
                    );
                }
                CeremonyStep::Three => {
                    assert!(state.org_key_pair_bundle.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<OrganizationKeyPairsBundle>
                    ));
                    assert!(
                        state.journalist_provisioning_key_pair_bundle.is_some_and(
                            exists_on_disk_and_can_be_deserialized::<
                                JournalistProvisioningKeyPairBundle,
                            >
                        )
                    );
                    assert!(state.covernode_provisioning_key_pair_bundle.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<CoverNodeProvisioningKeyPairBundle>
                    ));
                }
                CeremonyStep::Four => {
                    assert!(state.org_key_pair_bundle.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<OrganizationKeyPairsBundle>
                    ));
                    assert!(
                        state.journalist_provisioning_key_pair_bundle.is_some_and(
                            exists_on_disk_and_can_be_deserialized::<
                                JournalistProvisioningKeyPairBundle,
                            >
                        )
                    );
                    assert!(state.covernode_provisioning_key_pair_bundle.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<CoverNodeProvisioningKeyPairBundle>
                    ));

                    for covernode_id in 1..=(state.covernode_count).into() {
                        let covernode_identity = CoverNodeIdentity::from_node_id(covernode_id);
                        let covernode_database_path = state
                            .output_directory
                            .as_path()
                            .join(format!("{}.db", covernode_identity));
                        assert!(covernode_database_path.is_file())
                    }
                }
                CeremonyStep::Five => {
                    assert!(state.org_key_pair_bundle.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<OrganizationKeyPairsBundle>
                    ));
                    assert!(
                        state.journalist_provisioning_key_pair_bundle.is_some_and(
                            exists_on_disk_and_can_be_deserialized::<
                                JournalistProvisioningKeyPairBundle,
                            >
                        )
                    );
                    assert!(state.covernode_provisioning_key_pair_bundle.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<CoverNodeProvisioningKeyPairBundle>
                    ));
                    assert!(state
                        .admin_key_pair_bundle
                        .is_some_and(exists_on_disk_and_can_be_deserialized::<AdminKeyPairBundle>));
                }
                CeremonyStep::Six => {
                    assert!(state.org_key_pair_bundle.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<OrganizationKeyPairsBundle>
                    ));
                    assert!(
                        state.journalist_provisioning_key_pair_bundle.is_some_and(
                            exists_on_disk_and_can_be_deserialized::<
                                JournalistProvisioningKeyPairBundle,
                            >
                        )
                    );
                    assert!(state.covernode_provisioning_key_pair_bundle.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<CoverNodeProvisioningKeyPairBundle>
                    ));
                    assert!(state
                        .admin_key_pair_bundle
                        .is_some_and(exists_on_disk_and_can_be_deserialized::<AdminKeyPairBundle>));
                    assert!(state.set_system_status_available_bundle.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<SetSystemStatusAvailableBundle>
                    ));
                }
                CeremonyStep::Seven => {
                    assert!(state.org_key_pair_bundle.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<OrganizationKeyPairsBundle>
                    ));
                    assert!(
                        state.journalist_provisioning_key_pair_bundle.is_some_and(
                            exists_on_disk_and_can_be_deserialized::<
                                JournalistProvisioningKeyPairBundle,
                            >
                        )
                    );
                    assert!(state.covernode_provisioning_key_pair_bundle.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<CoverNodeProvisioningKeyPairBundle>
                    ));
                    assert!(state
                        .admin_key_pair_bundle
                        .is_some_and(exists_on_disk_and_can_be_deserialized::<AdminKeyPairBundle>));
                    assert!(state.set_system_status_available_bundle.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<SetSystemStatusAvailableBundle>
                    ));
                    assert!(state.anchor_org_pk_bundle.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<AnchorOrganizationPublicKeyBundle>
                    ));
                }
                CeremonyStep::Eight | CeremonyStep::Nine => {
                    assert!(state.org_key_pair_bundle.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<OrganizationKeyPairsBundle>
                    ));
                    assert!(
                        state.journalist_provisioning_key_pair_bundle.is_some_and(
                            exists_on_disk_and_can_be_deserialized::<
                                JournalistProvisioningKeyPairBundle,
                            >
                        )
                    );
                    assert!(state.covernode_provisioning_key_pair_bundle.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<CoverNodeProvisioningKeyPairBundle>
                    ));
                    assert!(state
                        .admin_key_pair_bundle
                        .is_some_and(exists_on_disk_and_can_be_deserialized::<AdminKeyPairBundle>));
                    assert!(state.set_system_status_available_bundle.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<SetSystemStatusAvailableBundle>
                    ));
                    assert!(state.anchor_org_pk_bundle.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<AnchorOrganizationPublicKeyBundle>
                    ));
                    assert!(state.public_key_forms_bundle.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<PublicKeyFormsBundle>
                    ));
                }
            }
        }
    }

    #[tokio::test]
    async fn when_running_the_ceremony_with_three_covernodes_generate_three_identity_keys() {
        let temp_dir = tempdir_in(std::env::current_dir().unwrap()).unwrap();
        // Three covernodes
        let covernodes = 3;
        let covernode_count = NonZeroU8::new(covernodes).unwrap();
        let covernode_db_password = "some-password";

        let mut ceremony = CeremonyStep::new();
        let mut state = CeremonyState::new(
            &temp_dir,
            covernode_count,
            true,
            covernode_db_password.to_string(),
            None,
            time::now(),
        );

        while let Some(step) = ceremony.next() {
            ceremony = step;
            ceremony.execute(&mut state).await.expect("Execute step");
        }

        let mut covernode_keys = HashMap::<CoverNodeIdentity, UntrustedCoverNodeIdKeyPair>::new();
        for covernode_id in 1..=(covernode_count).into() {
            // Check if there is db file for covernode identity
            let covernode_identity = CoverNodeIdentity::from_node_id(covernode_id);
            let covernode_database_path = state
                .output_directory
                .as_path()
                .join(format!("{}.db", covernode_identity));
            assert!(covernode_database_path.is_file());

            let db = Database::open(&covernode_database_path, covernode_db_password)
                .await
                .expect("Open database file");

            if let Some(covernode_key) = db.select_setup_bundle().await.expect("msg") {
                covernode_keys.insert(covernode_identity, covernode_key.1.key_pair);
            }
        }

        // Check there are 3 identity keys
        assert_eq!(covernode_keys.keys().count(), covernodes as usize);

        // Check that all keys are different
        assert_eq!(
            covernode_keys
                .values()
                .map(|v| v.public_key.clone())
                .collect::<HashSet<_>>()
                .len(),
            covernodes as usize
        );
    }

    #[tokio::test]
    async fn use_provided_organization_key_pair() {
        let temp_dir = tempdir_in(std::env::current_dir().unwrap()).unwrap();
        let covernode_count = NonZeroU8::new(1).unwrap();

        let now = time::now();
        generate_organization_key_pair(temp_dir.path(), true, now)
            .expect("Create new org key pair in temp dir");

        let original_org_key_pair = load_org_key_pairs(temp_dir.path(), now)
            .expect("Read back org key pair")[0]
            .to_untrusted();

        let mut ceremony = CeremonyStep::new();
        let mut state = CeremonyState::new(
            &temp_dir,
            covernode_count,
            true,
            "some-password".to_string(),
            Some(temp_dir.path().to_owned()),
            time::now(),
        );

        while let Some(step) = ceremony.next() {
            ceremony = step;
            ceremony
                .execute(&mut state)
                .await
                .expect("Failed to execute step");

            let state = state.clone();
            match ceremony {
                CeremonyStep::Zero => {
                    assert!(state.org_key_pair_bundle.is_none());
                }
                CeremonyStep::One => {
                    // Assert all bundles are there and can be deserialized
                    let org_key_pair_bundle_path = state
                        .org_key_pair_bundle
                        .expect("Org key pair bundle to be created");

                    // Get org key pair out of bundle
                    let reader = File::open(org_key_pair_bundle_path).expect("Could not open file");
                    let bundle = serde_json::from_reader::<_, OrganizationKeyPairsBundle>(reader)
                        .expect("Deserialize org key pair bundle");

                    // Check they are the same (the ceremony didn't generate a new key pair)
                    assert_eq!(bundle.org_key_pair, original_org_key_pair);
                }
                _ => {}
            }
        }
    }
}
