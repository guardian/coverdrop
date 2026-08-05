use chrono::Duration;
use common::{
    api::models::sentinel_id::SentinelIdentity,
    clap::Stage,
    crypto::keys::public_key::PublicKey,
    protocol::constants::{SENTINEL_ID_KEY_ROTATE_AFTER, SENTINEL_ID_KEY_VALID_DURATION},
};
use coverdrop_service::JournalistCoverDropService;
use integration_tests::{
    api_wrappers::{generate_test_journalist, get_and_verify_public_keys},
    secrets::MAILBOX_PASSWORD,
    stack::{CoverDropStack, StackProfile},
};
use journalist_vault::JournalistVault;

/// Test that sentinel ID key rotation works end-to-end.
///
/// Creates a journalist (which also creates a sentinel id key), time-travels past the
/// rotation threshold, calls `check_and_rotate_keys`, and asserts:
/// - There is a new latest sentinel ID key pair in the vault
/// - The new key is published to the API
/// - The old key is still present in the API (not yet expired)
/// Then time-travels past the validity duration of the initial key, calls `clean_up`, and asserts:
/// - The expired key is removed from the vault
/// - The expired key is removed from the API
#[tokio::test]
async fn sentinel_id_key_rotation() {
    pretty_env_logger::try_init().unwrap();

    let mut stack = CoverDropStack::builder(StackProfile::CoverDropOnly)
        .build()
        .await;

    let start_time = stack.now();

    let anchor_org_pks = stack.keys().anchor_org_pks();

    generate_test_journalist(
        stack.api_client_uncached(),
        stack.keys_path(),
        stack.temp_dir_path(),
        stack.now(),
        None,
        None,
    )
    .await;

    let vault_path = stack
        .temp_dir_path()
        .join("generated_test_journalist.vault");

    let vault = JournalistVault::open(&vault_path, MAILBOX_PASSWORD, Stage::Development)
        .await
        .expect("Load journalist vault");

    let expected_sentinel_id = SentinelIdentity::new("generated_test_journalist_sentinel").unwrap();

    // Confirm initial state: one sentinel ID key in API and vault
    let keys_and_profiles =
        get_and_verify_public_keys(stack.api_client_uncached(), &anchor_org_pks, stack.now()).await;

    let initial_sentinel_id_pks: Vec<_> = keys_and_profiles
        .keys
        .sentinel_id_pk_iter()
        .filter(|(sid, _)| *sid == &expected_sentinel_id)
        .collect();
    assert_eq!(
        initial_sentinel_id_pks.len(),
        1,
        "Should have one initial sentinel ID key"
    );

    let initial_vault_key_pair = vault
        .latest_sentinel_id_key_pair(stack.now())
        .await
        .expect("Get sentinel ID key pair")
        .expect("Sentinel ID key pair should exist");

    let initial_pk_hex = initial_vault_key_pair.public_key_hex();

    // Time travel past the rotation threshold
    stack
        .time_travel(stack.now() + SENTINEL_ID_KEY_ROTATE_AFTER + Duration::hours(1))
        .await;

    // Run check_and_rotate_keys which should trigger sentinel ID key rotation
    let journalist_coverdrop_service =
        JournalistCoverDropService::new(stack.api_client_uncached(), &vault);
    let did_rotate = journalist_coverdrop_service
        .check_and_rotate_keys(stack.now())
        .await
        .expect("check_and_rotate_keys should succeed");

    assert!(did_rotate, "Should have rotated at least one key");

    // Confirm the vault now has a new sentinel ID key pair
    let new_vault_key_pair = vault
        .latest_sentinel_id_key_pair(stack.now())
        .await
        .expect("Get sentinel ID key pair after rotation")
        .expect("Sentinel ID key pair should exist after rotation");

    let new_pk_hex = new_vault_key_pair.public_key_hex();
    assert_ne!(
        initial_pk_hex, new_pk_hex,
        "Sentinel ID key should have changed after rotation"
    );

    // Confirm the new key is published to the API
    let keys_and_profiles =
        get_and_verify_public_keys(stack.api_client_uncached(), &anchor_org_pks, stack.now()).await;

    let sentinel_id_pks: Vec<_> = keys_and_profiles
        .keys
        .sentinel_id_pk_iter()
        .filter(|(sid, _)| *sid == &expected_sentinel_id)
        .collect();

    // Both old and new keys should be present (old hasn't expired yet)
    assert_eq!(
        sentinel_id_pks.len(),
        2,
        "Should have two sentinel ID keys after rotation (old + new)"
    );

    // The new key from the vault should be among the API keys
    assert!(
        sentinel_id_pks
            .iter()
            .any(|(_, pk)| pk.public_key_hex() == new_pk_hex),
        "New sentinel ID key should be present in the API"
    );

    // The old key should still be present too
    assert!(
        sentinel_id_pks
            .iter()
            .any(|(_, pk)| pk.public_key_hex() == initial_pk_hex),
        "Old sentinel ID key should still be present in the API"
    );

    // Time travel past the validity duration of the initial key so it expires
    // The initial key was created at stack start_time, so we need to travel
    // past (start_time + SENTINEL_ID_KEY_VALID_DURATION)
    stack
        .time_travel(start_time + SENTINEL_ID_KEY_VALID_DURATION + Duration::hours(1))
        .await;

    // Run clean_up to delete expired keys from the vault
    vault
        .clean_up(stack.now())
        .await
        .expect("clean_up should succeed");

    // The expired (initial) key should be gone from the vault
    let vault_key_pairs_after_cleanup = vault
        .sentinel_id_key_pairs(stack.now())
        .await
        .expect("Get sentinel ID key pair after cleanup")
        .collect::<Vec<_>>();

    assert_eq!(
        vault_key_pairs_after_cleanup.len(),
        1,
        "Should have one sentinel ID key pair in the vault after cleanup (expired one removed)"
    );
    assert_eq!(
        vault_key_pairs_after_cleanup[0].public_key_hex(),
        new_pk_hex,
        "Latest vault key should be the rotated key, not the expired initial key"
    );

    // The expired key should also be gone from the API
    let keys_and_profiles =
        get_and_verify_public_keys(stack.api_client_uncached(), &anchor_org_pks, stack.now()).await;

    let sentinel_id_pks_after_expiry: Vec<_> = keys_and_profiles
        .keys
        .sentinel_id_pk_iter()
        .filter(|(sid, _)| *sid == &expected_sentinel_id)
        .collect();

    // Only the new key should remain (old one expired)
    assert_eq!(
        sentinel_id_pks_after_expiry.len(),
        1,
        "Should have only one sentinel ID key after expiry (old one gone)"
    );

    assert_eq!(
        sentinel_id_pks_after_expiry[0].1.public_key_hex(),
        new_pk_hex,
        "Remaining API key should be the rotated key"
    );
}
