use integration_tests::CoverDropStack;

#[tokio::test]
/// This test verifies the key counting logic for backups in the journalist vault.
async fn backup_keys_counting() {
    pretty_env_logger::try_init().unwrap();

    // generated_test_desk in the identity which we are backing up the vault for
    let default_journalist_id = "generated_test_desk";
    let stack = CoverDropStack::builder()
        .with_default_journalist_id(default_journalist_id)
        .build()
        .await;
    let api_client = stack.api_client_uncached();

    // load the journalist vault
    let journalist_vault = stack.load_static_journalist_vault().await;

    // Initially, there should be some keys since the (non-existing) last backup
    let count_0 = journalist_vault
        .get_count_of_keys_created_since_last_backup()
        .await
        .expect("Count keys since last backup");
    assert!(count_0 > 0, "Expected some new keys since last backup");

    // Simulate creating a backup by recording the current time as the last backup time
    journalist_vault
        .record_successful_backup(stack.now(), "/some/path/to/backup.vault")
        .await
        .expect("Record backup timestamp");

    // After recording a backup, there should be no new keys
    let count_1 = journalist_vault
        .get_count_of_keys_created_since_last_backup()
        .await
        .expect("Count keys since last backup");
    assert_eq!(count_1, 0, "Expected no new keys since last backup");

    // Rotating the messaging key should increase the count
    journalist_vault
        .generate_msg_key_pair_and_upload_pk(&api_client, stack.now())
        .await
        .expect("Generate and upload new messaging key");
    let count_2 = journalist_vault
        .get_count_of_keys_created_since_last_backup()
        .await
        .expect("Count keys since last backup");
    assert_eq!(
        count_2, 1,
        "Expected one new key since last backup after rotating messaging key"
    );

    // Creating and publishing a new identity key should further increase the count
    journalist_vault
        .generate_id_key_pair_and_rotate_pk(&api_client, stack.now())
        .await
        .expect("Generate and upload new ID key");
    let count_3 = journalist_vault
        .get_count_of_keys_created_since_last_backup()
        .await
        .expect("Count keys since last backup");
    assert_eq!(
        count_3, 2,
        "Expected two new keys since last backup after rotating ID key"
    );

    // Simulate another backup
    journalist_vault
        .record_successful_backup(stack.now(), "/some/path/to/backup2.vault")
        .await
        .expect("Record backup timestamp");

    // After the second backup, there should be no new keys
    let count_4 = journalist_vault
        .get_count_of_keys_created_since_last_backup()
        .await
        .expect("Count keys since last backup");
    assert_eq!(
        count_4, 0,
        "Expected no new keys since last backup after second backup"
    );
}
