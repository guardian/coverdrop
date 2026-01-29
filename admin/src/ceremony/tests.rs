#[cfg(test)]
mod test {
    use crate::ceremony::public_key_forms_bundle::PublicKeyFormsBundle;
    use crate::ceremony::set_system_status_available_bundle::SetSystemStatusAvailableBundle;
    use crate::ceremony::state_machine::CeremonyState;
    use crate::ceremony::{AssumeYes, CeremonyStep};
    use crate::CeremonyType;
    use common::api::models::covernode_id::CoverNodeIdentity;
    use common::backup::keys::{UntrustedBackupIdKeyPair, UntrustedBackupMsgKeyPair};
    use common::protocol::keys::{
        UntrustedCoverNodeIdKeyPair, UntrustedCoverNodeProvisioningKeyPair,
        UntrustedJournalistProvisioningKeyPair, UntrustedOrganizationPublicKey,
    };
    use common::system::keys::UntrustedAdminKeyPair;
    use common::time;
    use covernode_database::Database;
    use serde::de::DeserializeOwned;
    use std::collections::{HashMap, HashSet};
    use std::path::Path;
    use std::{fs::File, num::NonZeroU8};
    use strum::IntoEnumIterator;
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

        let mut state = CeremonyState::new(
            CeremonyType::InitialSetup,
            AssumeYes::DefaultYes,
            &temp_dir,
            Some(covernode_count),
            Some("some-password".to_string()),
            None,
            time::now(),
        );

        for step in CeremonyStep::iter() {
            step.execute(&mut state).await.expect("Execute step");
            let state = state.clone();
            match step {
                CeremonyStep::InitialStep => {}
                CeremonyStep::GenerateOrganizationKeyPair => {}
                CeremonyStep::GenerateJournalistProvisioningKeyPair => {
                    assert!(state.journalist_provisioning_key_pair_file.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<
                            UntrustedJournalistProvisioningKeyPair,
                        >
                    ));
                }
                CeremonyStep::GenerateCoverNodeProvisioningKeyPair => {
                    assert!(state.journalist_provisioning_key_pair_file.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<
                            UntrustedJournalistProvisioningKeyPair,
                        >
                    ));
                    assert!(state.covernode_provisioning_key_pair_file.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<
                            UntrustedCoverNodeProvisioningKeyPair,
                        >
                    ));
                }
                CeremonyStep::GenerateCoverNodeDatabase => {
                    assert!(state.journalist_provisioning_key_pair_file.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<
                            UntrustedJournalistProvisioningKeyPair,
                        >
                    ));
                    assert!(state.covernode_provisioning_key_pair_file.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<
                            UntrustedCoverNodeProvisioningKeyPair,
                        >
                    ));

                    for covernode_id in 1..=(state.covernode_count).unwrap().into() {
                        let covernode_identity = CoverNodeIdentity::from_node_id(covernode_id);
                        let covernode_database_path = state
                            .output_directory
                            .as_path()
                            .join(format!("{covernode_identity}.db"));
                        assert!(covernode_database_path.is_file())
                    }
                }
                CeremonyStep::GenerateAdminKeyPair => {
                    assert!(state.journalist_provisioning_key_pair_file.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<
                            UntrustedJournalistProvisioningKeyPair,
                        >
                    ));
                    assert!(state.covernode_provisioning_key_pair_file.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<
                            UntrustedCoverNodeProvisioningKeyPair,
                        >
                    ));
                    assert!(state.admin_key_pair_file.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<UntrustedAdminKeyPair>
                    ));
                }
                CeremonyStep::GenerateBackupKeys => {
                    assert!(state.journalist_provisioning_key_pair_file.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<
                            UntrustedJournalistProvisioningKeyPair,
                        >
                    ));
                    assert!(state.covernode_provisioning_key_pair_file.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<
                            UntrustedCoverNodeProvisioningKeyPair,
                        >
                    ));
                    assert!(state.admin_key_pair_file.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<UntrustedAdminKeyPair>
                    ));
                    assert!(state.backup_id_key_pair_file.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<UntrustedBackupIdKeyPair>
                    ));
                    assert!(state.backup_msg_key_pair_file.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<UntrustedBackupMsgKeyPair>
                    ));
                }
                CeremonyStep::GenerateSystemStatusBundle => {
                    assert!(state.journalist_provisioning_key_pair_file.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<
                            UntrustedJournalistProvisioningKeyPair,
                        >
                    ));
                    assert!(state.covernode_provisioning_key_pair_file.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<
                            UntrustedCoverNodeProvisioningKeyPair,
                        >
                    ));
                    assert!(state.admin_key_pair_file.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<UntrustedAdminKeyPair>
                    ));
                    assert!(state.backup_id_key_pair_file.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<UntrustedBackupIdKeyPair>
                    ));
                    assert!(state.backup_msg_key_pair_file.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<UntrustedBackupMsgKeyPair>
                    ));
                    assert!(state.set_system_status_available_bundle.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<SetSystemStatusAvailableBundle>
                    ));
                }
                CeremonyStep::GenerateAnchorOrganizationPublicKeyFile => {
                    assert!(state.journalist_provisioning_key_pair_file.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<
                            UntrustedJournalistProvisioningKeyPair,
                        >
                    ));
                    assert!(state.covernode_provisioning_key_pair_file.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<
                            UntrustedCoverNodeProvisioningKeyPair,
                        >
                    ));
                    assert!(state.admin_key_pair_file.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<UntrustedAdminKeyPair>
                    ));
                    assert!(state.backup_id_key_pair_file.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<UntrustedBackupIdKeyPair>
                    ));
                    assert!(state.backup_msg_key_pair_file.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<UntrustedBackupMsgKeyPair>
                    ));
                    assert!(state.set_system_status_available_bundle.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<SetSystemStatusAvailableBundle>
                    ));
                    assert!(state.anchor_org_pk_file.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<UntrustedOrganizationPublicKey>
                    ));
                }
                CeremonyStep::GeneratePublicKeyFormsBundle | CeremonyStep::FinalStep => {
                    assert!(state.journalist_provisioning_key_pair_file.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<
                            UntrustedJournalistProvisioningKeyPair,
                        >
                    ));
                    assert!(state.covernode_provisioning_key_pair_file.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<
                            UntrustedCoverNodeProvisioningKeyPair,
                        >
                    ));
                    assert!(state.admin_key_pair_file.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<UntrustedAdminKeyPair>
                    ));
                    assert!(state.backup_id_key_pair_file.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<UntrustedBackupIdKeyPair>
                    ));
                    assert!(state.backup_msg_key_pair_file.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<UntrustedBackupMsgKeyPair>
                    ));
                    assert!(state.set_system_status_available_bundle.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<SetSystemStatusAvailableBundle>
                    ));
                    assert!(state.anchor_org_pk_file.is_some_and(
                        exists_on_disk_and_can_be_deserialized::<UntrustedOrganizationPublicKey>
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

        let mut state = CeremonyState::new(
            CeremonyType::InitialSetup,
            AssumeYes::DefaultYes,
            &temp_dir,
            Some(covernode_count),
            Some(covernode_db_password.to_string()),
            None,
            time::now(),
        );

        for step in CeremonyStep::iter() {
            step.execute(&mut state).await.expect("Execute step");
        }

        let mut covernode_keys = HashMap::<CoverNodeIdentity, UntrustedCoverNodeIdKeyPair>::new();
        for covernode_id in 1..=(covernode_count).into() {
            // Check if there is db file for covernode identity
            let covernode_identity = CoverNodeIdentity::from_node_id(covernode_id);
            let covernode_database_path = state
                .output_directory
                .as_path()
                .join(format!("{covernode_identity}.db"));
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
}
