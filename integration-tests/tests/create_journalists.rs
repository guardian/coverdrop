use admin::{delete_journalist_form, submit_delete_journalist_form};
use chrono::Duration;
use common::protocol::constants::JOURNALIST_MSG_KEY_VALID_DURATION_SECONDS;
use integration_tests::{
    api_wrappers::{
        generate_test_desk, generate_test_journalist, get_and_verify_public_keys, get_public_keys,
        upload_new_messaging_key,
    },
    save_test_vector,
    secrets::MAILBOX_PASSWORD,
    CoverDropStack,
};
use journalist_vault::JournalistVault;

/// This tests that we have the correct initial state when we create a stack, and that
/// adding journalists works as expected.
///
/// Additionally it also checks that journalist keys are correctly verified and expired.
#[tokio::test]
async fn create_journalists() {
    pretty_env_logger::try_init().unwrap();

    let default_journalist_id = "generated_test_desk";

    let mut stack = CoverDropStack::builder()
        .with_default_journalist_id(default_journalist_id)
        .build()
        .await;

    let anchor_org_pks = stack.keys().anchor_org_pks();

    //
    // Confirm clean initial state
    //

    let keys = get_public_keys(stack.api_client_cached()).await;

    // Confirm that is only one journalist (the static journalist) when we start
    assert_eq!(keys.journalist_profiles.len(), 1);

    // Confirm our default journalist is none (since we haven't added it yet)
    assert!(keys.default_journalist_id.is_none());

    //
    // Initial journalist creation by an admin
    //

    // Insert journalist into API
    generate_test_journalist(
        stack.api_client_cached(),
        stack.keys_path(),
        stack.temp_dir_path(),
        stack.now(),
    )
    .await;

    generate_test_desk(
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

    let process_vault_initialization = vault
        .process_vault_setup_bundle(stack.api_client_cached(), stack.now())
        .await;

    assert!(matches!(process_vault_initialization, Ok(false)));

    let journalist_id = vault.journalist_id().await.expect("Get journalist ID");

    // Now we've inserted our journalist, get the keys again
    let keys_and_profiles =
        get_and_verify_public_keys(stack.api_client_cached(), &anchor_org_pks, stack.now()).await;

    // Now we've inserted our desk we expect the default journalist to be set
    assert!(keys_and_profiles
        .default_journalist_id
        .is_some_and(
            |actual_default_journalist_id| &*actual_default_journalist_id == default_journalist_id
        ));

    assert_eq!(keys_and_profiles.journalist_profiles.len(), 3);

    // Confirm our journalist has an initial key
    assert_eq!(
        keys_and_profiles
            .keys
            .journalist_msg_pk_iter_for_identity(&journalist_id)
            .count(),
        1
    );
    assert_eq!(
        keys_and_profiles
            .keys
            .journalist_id_pk_iter_for_identity(&journalist_id)
            .count(),
        1
    );

    // After cleaning up the vault, keys should still exist
    vault.clean_up(stack.now()).await.expect("Clean up vault");

    let journalist_id_keys = vault.id_key_pairs(stack.now()).await.unwrap();
    assert!(journalist_id_keys.count() == 1);
    let journalist_msg_keys = vault
        .msg_key_pairs_for_decryption(stack.now())
        .await
        .unwrap();
    assert!(journalist_msg_keys.count() == 1);

    //
    // Journalist publishes a new messaging key
    //

    stack.time_travel(stack.now() + Duration::days(1)).await;

    upload_new_messaging_key(stack.api_client_cached(), &vault, stack.now()).await;
    let keys_and_profiles =
        get_and_verify_public_keys(stack.api_client_cached(), &anchor_org_pks, stack.now()).await;

    // Confirm there's still three journalists
    assert_eq!(keys_and_profiles.journalist_profiles.len(), 3);

    // Confirm our journalist has their initial key, and the new key we just uploaded
    assert_eq!(
        keys_and_profiles
            .keys
            .journalist_msg_pk_iter_for_identity(&journalist_id)
            .count(),
        2
    );
    assert_eq!(
        keys_and_profiles
            .keys
            .journalist_id_pk_iter_for_identity(&journalist_id)
            .count(),
        1
    );

    // Emit state before time travel: keys are valid
    save_test_vector!("initial_state", &stack);

    //
    // Check API still displays keys as they expire
    //

    // Add a minute to `now` to side step any weird precision issues
    // when checking the certificates
    let post_expiry = stack.now()
        + Duration::minutes(1)
        + Duration::seconds(JOURNALIST_MSG_KEY_VALID_DURATION_SECONDS);
    stack.time_travel(post_expiry).await;

    let keys_and_profiles =
        get_and_verify_public_keys(stack.api_client_cached(), &anchor_org_pks, stack.now()).await;

    // Confirm there's still three journalists
    assert_eq!(keys_and_profiles.journalist_profiles.len(), 3);

    assert_eq!(
        keys_and_profiles
            .keys
            .journalist_msg_pk_iter_for_identity(&journalist_id)
            .count(),
        0
    );

    // Emit state after initial time travel: keys are expired but still displayed
    save_test_vector!("post_expiry_still_displayed", &stack);

    //
    // Check API doesn't display keys 7 days after they're no longer valid (21 days after they were created)
    //

    let post_display = stack.now() + Duration::days(7);
    stack.time_travel(post_display).await;

    // Clean up the vault
    vault.clean_up(stack.now()).await.expect("Clean up vault");

    // expired msg keys should have been deleted, id key still exists
    let journalist_id_keys = vault.id_key_pairs(stack.now()).await.unwrap();
    assert!(journalist_id_keys.count() == 1);
    let journalist_msg_keys = vault
        .msg_key_pairs_for_decryption(stack.now())
        .await
        .unwrap();
    assert!(journalist_msg_keys.count() == 0);

    let keys_and_profiles =
        get_and_verify_public_keys(stack.api_client_cached(), &anchor_org_pks, stack.now()).await;

    // Confirm there's still three journalists
    assert_eq!(keys_and_profiles.journalist_profiles.len(), 3);

    // Confirm our journalist's keys are all gone (expired)
    assert_eq!(
        keys_and_profiles
            .keys
            .journalist_msg_pk_iter_for_identity(&journalist_id)
            .count(),
        0
    );

    // Emit state after second time travel: keys are expired so long that they are no longer displayed
    save_test_vector!("post_display", &stack);

    // after 8 weeks, the id key is also deleted
    let post_garbage_collect_id_key = stack.now() + Duration::weeks(5);
    stack.time_travel(post_garbage_collect_id_key).await;
    vault.clean_up(stack.now()).await.expect("Clean up vault");

    // all keys deleted
    let journalist_id_keys = vault.id_key_pairs(stack.now()).await.unwrap();
    assert!(journalist_id_keys.count() == 0);
    let journalist_msg_keys = vault
        .msg_key_pairs_for_decryption(stack.now())
        .await
        .unwrap();
    assert!(journalist_msg_keys.count() == 0);

    //
    // Delete journalist
    //
    // Normally this is done by creating the deletion form offline with a journlist
    // provisioning key and then the form is submitted separately so that the key
    // material is never exposed to the internet.
    //

    let delete_form_path = delete_journalist_form(
        stack.keys_path(),
        &journalist_id,
        stack.temp_dir_path(),
        stack.now(),
    )
    .await
    .expect("Create journalist deletion form");

    submit_delete_journalist_form(stack.api_client_uncached(), delete_form_path)
        .await
        .expect("Delete journalist submission");

    let keys_and_profiles =
        get_and_verify_public_keys(stack.api_client_cached(), &anchor_org_pks, stack.now()).await;

    assert_eq!(keys_and_profiles.journalist_profiles.len(), 2);
    assert!(!keys_and_profiles
        .journalist_profiles
        .iter()
        .any(|profile| profile.id == journalist_id));

    assert!(!stack.do_secrets_exist_in_stack().await);
}
