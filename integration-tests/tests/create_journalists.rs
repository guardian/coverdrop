use admin::{delete_journalist_form, submit_delete_journalist_form};
use chrono::Duration;

use common::api::models::sentinel_id::SentinelIdentity;
use common::clap::Stage;
use common::crypto::keys::public_key::PublicKey;
use common::protocol::constants::JOURNALIST_MSG_KEY_VALID_DURATION;
use coverdrop_service::{JournalistCoverDropService, ProcessVaultSetupBundleResult};
use integration_tests::{
    api_wrappers::{
        generate_test_desk, generate_test_journalist, get_and_verify_public_keys, get_public_keys,
        upload_new_messaging_key,
    },
    save_test_vector,
    secrets::MAILBOX_PASSWORD,
    stack::{CoverDropStack, StackProfile},
};
use journalist_vault::JournalistVault;

/// This tests that we have the correct initial state when we create a stack, and that
/// adding journalists works as expected.
///
/// It also checks that journalist and sentinel keys are correctly verified,
/// stored in the vault, published to the API, and expired.
#[tokio::test]
async fn create_journalists() {
    integration_tests::utils::init_logger();

    let default_journalist_id = "generated_test_desk";

    let mut stack = CoverDropStack::builder(StackProfile::CoverDropOnly)
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
        None,
        None,
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

    let vault = JournalistVault::open(&vault_path, MAILBOX_PASSWORD, Stage::Development)
        .await
        .expect("Load journalist vault");

    let service = JournalistCoverDropService::new(stack.api_client_cached(), &vault);
    let process_vault_initialization = service.process_vault_setup_bundle(stack.now()).await;

    assert!(matches!(
        process_vault_initialization,
        Ok(ProcessVaultSetupBundleResult::AlreadyRegistered)
    ));

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

    // Confirm sentinel profile was registered in the API
    let expected_sentinel_id = SentinelIdentity::new("generated_test_journalist_sentinel").unwrap();
    assert!(
        keys_and_profiles
            .sentinel_profiles
            .iter()
            .any(|sp| sp.id == expected_sentinel_id),
        "Expected sentinel profile for generated_test_journalist_sentinel"
    );

    // Confirm sentinel ID key was registered in the API
    let (_, api_sentinel_id_pk) = keys_and_profiles
        .keys
        .sentinel_id_pk_iter()
        .find(|(sid, _)| *sid == &expected_sentinel_id)
        .expect("Sentinel ID public key should exist in API");

    // Confirm sentinel identity and key pair are stored in the vault
    let vault_sentinel_id = vault
        .sentinel_id()
        .await
        .expect("Get sentinel ID from vault");
    assert_eq!(vault_sentinel_id.as_ref(), Some(&expected_sentinel_id));

    let vault_sentinel_id_key_pair = vault
        .latest_sentinel_id_key_pair(stack.now())
        .await
        .expect("Get sentinel ID key pair from vault")
        .expect("Sentinel ID key pair should exist in vault");

    // Confirm the public key from the API matches the one in the vault
    assert_eq!(
        api_sentinel_id_pk.public_key_hex(),
        vault_sentinel_id_key_pair.public_key_hex(),
        "Sentinel ID public key in API should match the one in the vault"
    );

    // After cleaning up the vault, keys should still exist
    vault.clean_up(stack.now()).await.expect("Clean up vault");

    let journalist_id_keys = vault.journalist_id_key_pairs(stack.now()).await.unwrap();
    assert_eq!(journalist_id_keys.count(), 1);
    let journalist_msg_keys = vault
        .msg_key_pairs_for_decryption(stack.now())
        .await
        .unwrap();
    assert_eq!(journalist_msg_keys.count(), 1);

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
    let post_expiry = stack.now() + Duration::minutes(1) + JOURNALIST_MSG_KEY_VALID_DURATION;
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
    let journalist_id_keys = vault.journalist_id_key_pairs(stack.now()).await.unwrap();
    assert_eq!(journalist_id_keys.count(), 1);
    let journalist_msg_keys = vault
        .msg_key_pairs_for_decryption(stack.now())
        .await
        .unwrap();
    assert_eq!(journalist_msg_keys.count(), 0);

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
    let journalist_id_keys = vault.journalist_id_key_pairs(stack.now()).await.unwrap();
    assert_eq!(journalist_id_keys.count(), 0);
    let journalist_msg_keys = vault
        .msg_key_pairs_for_decryption(stack.now())
        .await
        .unwrap();
    assert_eq!(journalist_msg_keys.count(), 0);

    // sentinel id key should also be expired and deleted
    let sentinel_id_key_pair = vault
        .latest_sentinel_id_key_pair(stack.now())
        .await
        .expect("Get sentinel ID key pair from vault");
    assert!(
        sentinel_id_key_pair.is_none(),
        "Sentinel ID key pair should be expired and deleted after 8+ weeks"
    );

    //
    // Delete journalist
    //
    // Normally this is done by creating the deletion form offline with a journalist
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
