use common::crypto::keys::serde::StorableKeyMaterial;
use common::protocol::backup::{coverup_finish_restore_step, coverup_initiate_restore_step};
use common::protocol::backup::{
    sentinel_restore_try_unwrap_and_wrap_share_step, WrappedSecretShare,
};
use common::{
    api::forms::{
        GetBackupDataForm, PostBackupDataForm, PostBackupIdKeyForm, PostBackupMsgKeyForm,
    },
    backup::{
        keys::{generate_backup_id_key_pair, generate_backup_msg_key_pair},
        roles::{BackupId, BackupMsg},
    },
    crypto::keys::{encryption::SignedEncryptionKeyPair, signing::SignedSigningKeyPair},
    protocol::{
        backup::{sentinel_create_backup, RecoveryContact},
        roles::JournalistMessaging,
    },
};
use integration_tests::api_wrappers::get_and_verify_public_keys;
use integration_tests::{
    api_wrappers::generate_test_journalist, secrets::MAILBOX_PASSWORD, CoverDropStack,
};
use journalist_vault::JournalistVault;
use std::fs;
use std::time::Duration;

#[tokio::test]
/// This test covers the creation and retrieval of a backup for a journalist's vault.
/// This does not cover the actual restoration of a backup but this will be added later.
async fn backup_scenario() {
    pretty_env_logger::try_init().unwrap();

    // generated_test_desk in the identity which we are backing up the vault for
    let default_journalist_id = "generated_test_desk";
    let stack = CoverDropStack::builder()
        .with_default_journalist_id(default_journalist_id)
        .build()
        .await;

    // temporary directory to store the recovery state
    let backup_recovery_dir = stack.temp_dir_path().join("backup_recovery");
    fs::create_dir_all(&backup_recovery_dir).expect("Create backup recovery dir");

    // Create a backup keypair for the sentinel to use to encrypt the backup and connect to the journalist vault
    let org_keypair = stack.keys().org_key_pair.clone();

    // Create and upload two sets of backup keys to ensure we can retrieve the latest one later
    let backup_signing_key = create_test_backup_id_key_pair(&stack);
    let backup_signing_key_2 = create_test_backup_id_key_pair(&stack);

    // This helper creates multiple backup message keys and uploads them to the API
    // This is to help test that we can retrieve the latest key later and also supports multiple messages keys per backup id key
    // It returns the last two keys created so we can use them later
    async fn upload_and_save_backup_keys(
        stack: &CoverDropStack,
        backup_signing_key: &SignedSigningKeyPair<BackupId>,
        org_keypair: &SignedSigningKeyPair<common::protocol::roles::Organization>,
    ) -> (
        SignedEncryptionKeyPair<BackupMsg>,
        SignedEncryptionKeyPair<BackupMsg>,
    ) {
        // Write backup keypair to disk for use by the CLI tool
        backup_signing_key
            .to_untrusted()
            .save_to_disk(stack.keys_path())
            .unwrap();

        // Upload the backup signing key
        let post_backup_signing_pk = PostBackupIdKeyForm::new(
            backup_signing_key.public_key().to_untrusted(),
            org_keypair,
            stack.now(),
        )
        .expect("Create PostBackupDataForm");

        stack
            .api_client_uncached()
            .post_backup_signing_pk(post_backup_signing_pk)
            .await
            .expect("Upload backup signing key");

        let backup_encryption_key_1 =
            create_test_backup_msg_key_pair(stack, backup_signing_key.clone());

        let backup_encryption_key_2 =
            create_test_backup_msg_key_pair(stack, backup_signing_key.clone());

        for key in [&backup_encryption_key_1, &backup_encryption_key_2] {
            // Upload the backup encryption key
            let post_backup_encryption_pk = PostBackupMsgKeyForm::new(
                key.public_key().to_untrusted(),
                backup_signing_key,
                stack.now(),
            )
            .expect("Create PostBackupMsgKeyForm");

            stack
                .api_client_uncached()
                .post_backup_encryption_pk(post_backup_encryption_pk)
                .await
                .expect("Upload backup encryption key");

            // Also save to disk for use by the CLI tool
            key.to_untrusted().save_to_disk(stack.keys_path()).unwrap();

            // Sleep a bit to ensure the keys have different timestamps
            tokio::time::sleep(Duration::from_millis(1000)).await;
        }
        (backup_encryption_key_1, backup_encryption_key_2)
    }

    let (_, _) = upload_and_save_backup_keys(&stack, &backup_signing_key, &org_keypair).await;

    let (_, backup_encryption_key_2b) =
        upload_and_save_backup_keys(&stack, &backup_signing_key_2, &org_keypair).await;

    // Create the journalist vault to back up
    let journalist_vault = stack.load_static_journalist_vault().await;

    // Extract the journalist identity and signing keypair
    let journalist_identity = journalist_vault.journalist_id().await.unwrap();

    let journalist_signing_pair = journalist_vault
        .latest_id_key_pair(stack.now())
        .await
        .unwrap()
        .unwrap();

    // Create recovery contact which is another journalist who can help recover the vault
    let (recovery_contact_vault, recovery_contact_messaging_pair) =
        create_recovery_contact_vault_and_return_messaging_keys(&stack).await;

    // Create the signed backup data
    let journalist_vault_bytes = stack.load_static_journalist_vault_bytes().await;
    let recovery_contact = RecoveryContact {
        identity: recovery_contact_vault.journalist_id().await.unwrap(),
        latest_messaging_key: recovery_contact_messaging_pair.public_key().clone(),
    };

    let anchor_org_pks = stack.keys().anchor_org_pks();

    // Get the backup keys from the api first to simulate a real world scenario
    let fresh_public_keys =
        get_and_verify_public_keys(stack.api_client_uncached(), &anchor_org_pks, stack.now())
            .await
            .keys;

    assert!(
        fresh_public_keys.backup_msg_pk_iter().next().is_some(),
        "No backup messaging keys found from API"
    );
    let backup_encryption_key_from_api = fresh_public_keys.latest_backup_msg_pk().unwrap();

    // Make sure we are getting the most recently uploaded backup key
    assert_eq!(
        backup_encryption_key_from_api,
        *backup_encryption_key_2b.public_key()
    );

    let verified_backup_data = sentinel_create_backup(
        journalist_vault_bytes.clone(),
        journalist_identity.clone(),
        journalist_signing_pair.clone(),
        backup_encryption_key_from_api.clone(),
        vec![recovery_contact],
        1, // k=1
        stack.now(),
    )
    .expect("Failed to create backup");

    assert!(!verified_backup_data.backup_data_bytes.0.is_empty());

    // Upload the signed backup data to the API
    let backup_form = PostBackupDataForm::new(
        verified_backup_data.clone().to_unverified().unwrap(),
        &journalist_signing_pair,
        stack.now(),
    )
    .expect("Create PostBackupDataForm");

    stack
        .api_client_uncached()
        .post_backup_data(backup_form.clone())
        .await
        .expect("Upload backup data");

    // Testing Duplicated Insert prevention - re-posting the backup should fail since we have a unique constraint on the data hash
    let result = stack
        .api_client_uncached()
        .post_backup_data(backup_form)
        .await;
    assert!(result.is_err());

    // Testing using different signing key between form and backup data - this should fail
    let incorrect_signing_key = recovery_contact_vault
        .latest_id_key_pair(stack.now())
        .await
        .unwrap()
        .unwrap();
    let backup_form = PostBackupDataForm::new(
        verified_backup_data.clone().to_unverified().unwrap(),
        &incorrect_signing_key,
        stack.now(),
    )
    .expect("Create PostBackupDataForm");

    let result = stack
        .api_client_uncached()
        .post_backup_data(backup_form.clone())
        .await;
    assert!(result.is_err());

    let journalist_identity_get_backup_form = GetBackupDataForm::new(
        journalist_identity.clone(),
        &backup_signing_key,
        stack.now(),
    )
    .expect("Create GetBackupDataForm");

    // Retrieve the backup data from the API
    let retrieved_signed_backup_data = stack
        .api_client_uncached()
        .get_backup_data(journalist_identity_get_backup_form)
        .await
        .expect("Failed to retrieve backup data");

    assert_eq!(
        verified_backup_data.to_unverified().unwrap(),
        retrieved_signed_backup_data
    );

    // Verify the retrieved backup data
    let verified_retrieved_signed_backup_data =
        retrieved_signed_backup_data.to_verified(journalist_signing_pair.public_key(), stack.now());

    assert!(verified_retrieved_signed_backup_data.is_ok());

    let verified_retrieved_signed_backup_data = verified_retrieved_signed_backup_data.unwrap();

    // Check the contents of the backup data matches what we originally created
    let retrieved_backup_data_bytes = verified_retrieved_signed_backup_data.backup_data().unwrap();

    // Initiate restore
    let backup_state = coverup_initiate_restore_step(
        journalist_identity.clone(),
        retrieved_backup_data_bytes
            .to_backup_data_with_signature(&journalist_signing_pair)
            .unwrap(),
        journalist_signing_pair.public_key(),
        std::slice::from_ref(&backup_encryption_key_2b),
        stack.now(),
    )
    .expect("Failed to initiate restore");

    // Recovery contact unwraps share
    let wrapped_share = sentinel_restore_try_unwrap_and_wrap_share_step(
        backup_state.encrypted_shares.clone(),
        vec![recovery_contact_messaging_pair.clone()],
        backup_encryption_key_2b.public_key().clone(),
    )
    .expect("Failed to unwrap share")
    .expect("No share could be unwrapped");

    // Complete restore
    let restored_vault = coverup_finish_restore_step(
        backup_state,
        vec![wrapped_share.clone()],
        &[backup_encryption_key_2b.clone()],
    )
    .expect("Failed to finish restore");

    // Verify the round-trip worked
    assert_eq!(journalist_vault_bytes.clone(), restored_vault);

    // Replace the vault file with the restored vault to verify it can be opened
    stack
        .save_static_journalist_vault_bytes(restored_vault)
        .await;

    let restored_vault = stack.load_static_journalist_vault().await;

    assert_eq!(
        restored_vault.journalist_id().await.unwrap(),
        journalist_identity
    );

    //
    // Also check the CLI tools can also retrieve the stored backup correctly
    //

    // Step 1: Prepare the backup restore bundle (offline/air-gapped)
    let prepare_bundle_path = admin::backup_initiate_restore_prepare(
        stack.keys_path().to_path_buf(),
        journalist_identity.clone(),
        &backup_recovery_dir,
        stack.now(),
    )
    .await
    .expect("Admin CLI initiate restore prepare");

    // Step 2: Submit the bundle to the API (online)
    let response_bundle_path = admin::backup_initiate_restore_submit(
        &prepare_bundle_path,
        stack.api_client_uncached().base_url.clone(),
        &backup_recovery_dir,
    )
    .await
    .expect("Admin CLI initiate restore submit");

    // Step 3: Finalize the restore process (offline/air-gapped)
    let (backup_output_path, wrapped_shares_paths) = admin::backup_initiate_restore_finalize(
        &response_bundle_path,
        stack.keys_path().to_path_buf(),
        &backup_recovery_dir,
        stack.now(),
    )
    .await
    .expect("Admin CLI initiate restore finalize");

    // Load the wrapped shares from disk
    let encrypted_shares = wrapped_shares_paths
        .iter()
        .map(|path| fs::read_to_string(path).expect("Read wrapped share file from disk"))
        .filter_map(|share_base64| WrappedSecretShare::from_base64_string(&share_base64).ok())
        .collect::<Vec<WrappedSecretShare>>();

    // Hand over to recovery contact to unwrap and rewrap the share
    let rewrapped_share = sentinel_restore_try_unwrap_and_wrap_share_step(
        encrypted_shares,
        vec![recovery_contact_messaging_pair],
        backup_encryption_key_2b.public_key().clone(),
    )
    .expect("Failed to unwrap share")
    .expect("No share could be unwrapped");

    let restored_vault_via_cli_path = admin::backup_complete_restore(
        &backup_output_path,
        &backup_recovery_dir,
        stack.keys_path(),
        vec![rewrapped_share],
        stack.now(),
    )
    .await
    .expect("Admin CLI complete restore");

    let restored_vault_via_cli_bytes =
        fs::read(restored_vault_via_cli_path).expect("Load restored journalist vault via cli");

    // Replace the vault file with the restored vault to verify it can be opened
    stack
        .save_static_journalist_vault_bytes(restored_vault_via_cli_bytes)
        .await;

    let restored_vault = stack.load_static_journalist_vault().await;

    assert_eq!(
        restored_vault.journalist_id().await.unwrap(),
        journalist_identity
    );
}

async fn create_recovery_contact_vault_and_return_messaging_keys(
    stack: &CoverDropStack,
) -> (
    JournalistVault,
    SignedEncryptionKeyPair<JournalistMessaging>,
) {
    // generated_test_journalist is a recovery contact for generated_test_desk
    generate_test_journalist(
        stack.api_client_cached(),
        stack.keys_path(),
        stack.temp_dir_path(),
        stack.now(),
    )
    .await;

    let vault_path = stack
        .temp_dir_path()
        .join("generated_test_journalist.vault");

    let vault = JournalistVault::open(&vault_path, MAILBOX_PASSWORD)
        .await
        .expect("Load journalist vault");

    let journalist_id_keys = vault
        .latest_msg_key_pair(stack.now())
        .await
        .unwrap()
        .unwrap();

    (vault, journalist_id_keys)
}

fn create_test_backup_msg_key_pair(
    stack: &CoverDropStack,
    signing_key_pair: SignedSigningKeyPair<BackupId>,
) -> SignedEncryptionKeyPair<BackupMsg> {
    generate_backup_msg_key_pair(&signing_key_pair, stack.now())
}

fn create_test_backup_id_key_pair(stack: &CoverDropStack) -> SignedSigningKeyPair<BackupId> {
    generate_backup_id_key_pair(&stack.keys().org_key_pair, stack.now())
}
