use chrono::Duration;
use common::form::DEFAULT_FORM_TTL;
use common::protocol::constants::JOURNALIST_PROVISIONING_KEY_ROTATE_AFTER;
use common::{
    api::{
        forms::{PostJournalistIdPublicKeyForm, PostJournalistProvisioningPublicKeyForm},
        models::journalist_id::JournalistIdentity,
    },
    protocol::keys::{
        generate_journalist_id_key_pair, generate_journalist_messaging_key_pair,
        generate_journalist_provisioning_key_pair,
    },
};
use integration_tests::{
    api_wrappers::{generate_test_journalist, get_and_verify_public_keys},
    secrets::MAILBOX_PASSWORD,
    CoverDropStack,
};
use itertools::Itertools;
use journalist_vault::JournalistVault;

/// This test asserts that expired journalist id key rotation forms are not returned by the API.
#[tokio::test]
async fn id_key_rotation_form_expires() {
    let mut stack = CoverDropStack::builder()
        // triggered identity api task runner so that journalist id key rotation task is only run when manually triggered
        .with_identity_api_task_runner_mode(common::task::RunnerMode::ManuallyTriggered)
        .build()
        .await;

    generate_test_journalist(
        stack.api_client_uncached(),
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

    // post new id key form to API
    vault
        .generate_id_key_pair_and_rotate_pk(stack.api_client_uncached(), stack.now())
        .await
        .expect("rotate id key pair");

    // the form should be present immediately after posting
    let id_key_rotation_forms = stack
        .api_client_uncached()
        .get_journalist_id_pk_forms()
        .await
        .expect("fetched id key rotation forms from API");
    assert_eq!(
        id_key_rotation_forms.len(),
        1,
        "id key rotation form should be present"
    );

    // time travel past id key form expiry
    stack
        .time_travel(stack.now() + DEFAULT_FORM_TTL + Duration::seconds(10))
        .await;

    // the form should have expired and is no longer returned by the API
    let id_key_rotation_forms = stack
        .api_client_uncached()
        .get_journalist_id_pk_forms()
        .await
        .expect("fetched id key rotation forms from API");
    assert!(
        id_key_rotation_forms.is_empty(),
        "id key rotation form should have expired"
    );
}

/// This test simulates a scenario where the journalist creates a candidate id key but fails to publish it before a
/// successful provisioning key rotation. The API should accept the candidate id key signed by the old provisioning key.
#[tokio::test]
async fn concurrent_journalist_id_and_provisioning_key_rotations() {
    let mut stack = CoverDropStack::builder().build().await;

    let anchor_org_pks = stack.keys().anchor_org_pks();

    generate_test_journalist(
        stack.api_client_uncached(),
        stack.keys_path(),
        stack.temp_dir_path(),
        stack.now(),
    )
    .await;

    // Assert initial keys created
    let keys =
        get_and_verify_public_keys(stack.api_client_uncached(), &anchor_org_pks, stack.now())
            .await
            .keys;
    assert_eq!(keys.journalist_provisioning_pk_iter().count(), 1);
    // starting with static_test_journalist and generated_test_journalist
    assert_eq!(keys.journalist_id_pk_iter().count(), 2);

    // time travel to avoid new provisioning key being rejected by api
    stack
        .time_travel(stack.now() + JOURNALIST_PROVISIONING_KEY_ROTATE_AFTER)
        .await;

    // Create, but don't publish, a new journalist id key signed with the current journalist provisioning key
    let journalist_provisioning_key_pair_1 = stack.keys().journalist_provisioning_key_pair.clone();

    let new_journalist_id_key_pair =
        generate_journalist_id_key_pair(&journalist_provisioning_key_pair_1, stack.now());

    // Rotate journalist provisioning key
    let journalist_provisioning_key_pair_2 =
        generate_journalist_provisioning_key_pair(&stack.keys().org_key_pair, stack.now());
    let journalist_provisioning_form = PostJournalistProvisioningPublicKeyForm::new(
        journalist_provisioning_key_pair_2.to_untrusted().public_key,
        &stack.keys().org_key_pair,
        stack.now(),
    )
    .expect("created journalist provisioning form");

    stack
        .api_client_cached()
        .post_journalist_provisioning_pk(journalist_provisioning_form)
        .await
        .expect("posted new provisioning key");

    // Assert new provisioning key included in API response
    let keys =
        get_and_verify_public_keys(stack.api_client_uncached(), &anchor_org_pks, stack.now())
            .await
            .keys;
    assert_eq!(keys.journalist_provisioning_pk_iter().count(), 2);
    assert_eq!(keys.journalist_id_pk_iter().count(), 0); // (initial id keys expired)

    // Post the new id key signed with the old provisioning key to the API
    stack
        .api_client_uncached()
        .post_journalist_id_pk_form(
            PostJournalistIdPublicKeyForm::new(
                JournalistIdentity::new("generated_test_journalist").unwrap(),
                new_journalist_id_key_pair.to_untrusted().public_key,
                false,
                &journalist_provisioning_key_pair_2, // the new provisioning key, not the one which signed the id key!
                stack.now(),
            )
            .expect("created journalist id form"),
        )
        .await
        .expect("posting id key should succeed");

    // Assert the new id key is included in API response
    let keys =
        get_and_verify_public_keys(stack.api_client_uncached(), &anchor_org_pks, stack.now())
            .await
            .keys;
    assert_eq!(keys.journalist_provisioning_pk_iter().count(), 2);
    assert_eq!(keys.journalist_id_pk_iter().count(), 1);

    // Assert that the new id key is in the hierarchy under the first provisioning key
    let provisioning_key_1_id_pks = keys.journalist_id_pk_iter_for_provisioning_pk(
        &journalist_provisioning_key_pair_1.public_key(),
    );
    assert_eq!(
        provisioning_key_1_id_pks
            .max_by_key(|pk| pk.not_valid_after)
            .unwrap(),
        new_journalist_id_key_pair.public_key()
    );
    // and there are no id keys under provisioning key 2
    let provisioning_key_2_id_pks = keys.journalist_id_pk_iter_for_provisioning_pk(
        &journalist_provisioning_key_pair_2.public_key(),
    );
    assert_eq!(provisioning_key_2_id_pks.collect_vec().len(), 0);
}

/// This test simulates a scenario where the journalist creates a candidate msg key but fails to publish it before a
/// successful identity key rotation. The API should accept the candidate msg key signed by the old identity key.
#[tokio::test]
async fn concurrent_journalist_msg_and_id_key_rotations() {
    // TODO this test only needs an api. Should builder have with_api, with_covernode, etc.
    // so we don't create an entire stack when we only need part of it?
    let stack = CoverDropStack::builder().build().await;

    let anchor_org_pks = stack.keys().anchor_org_pks();

    generate_test_journalist(
        stack.api_client_uncached(),
        stack.keys_path(),
        stack.temp_dir_path(),
        stack.now(),
    )
    .await;

    // Assert initial keys created
    let keys =
        get_and_verify_public_keys(stack.api_client_uncached(), &anchor_org_pks, stack.now())
            .await
            .keys;
    // starting with static_test_journalist and generated_test_journalist
    assert_eq!(keys.journalist_id_pk_iter().count(), 2);
    assert_eq!(keys.journalist_msg_pk_iter().count(), 2);

    // Create, but don't publish, a new journalist msg key signed with the current journalist id key
    let vault_path = stack
        .temp_dir_path()
        .join("generated_test_journalist.vault");

    let vault = JournalistVault::open(&vault_path, MAILBOX_PASSWORD)
        .await
        .expect("Load journalist vault");

    let journalist_id_key_pair_1 = vault
        .latest_id_key_pair(stack.now())
        .await
        .unwrap()
        .unwrap();

    let new_journalist_msg_key_pair =
        generate_journalist_messaging_key_pair(&journalist_id_key_pair_1, stack.now());

    // Rotate journalist id key
    let journalist_id_key_pair_2 = generate_journalist_id_key_pair(
        &stack.keys().journalist_provisioning_key_pair,
        stack.now(),
    );
    let journalist_id_form = PostJournalistIdPublicKeyForm::new(
        JournalistIdentity::new("generated_test_journalist").unwrap(),
        journalist_id_key_pair_2.to_untrusted().public_key,
        false,
        &stack.keys().journalist_provisioning_key_pair,
        stack.now(),
    )
    .expect("created journalist id form");

    stack
        .api_client_cached()
        .post_journalist_id_pk_form(journalist_id_form)
        .await
        .expect("posted new id key");

    // Assert new id key included in API response
    let keys =
        get_and_verify_public_keys(stack.api_client_uncached(), &anchor_org_pks, stack.now())
            .await
            .keys;
    assert_eq!(keys.journalist_id_pk_iter().count(), 3);
    assert_eq!(keys.journalist_msg_pk_iter().count(), 2);

    // Post the new msg key signed with the old id key to the API
    stack
        .api_client_uncached()
        .post_journalist_msg_pk(
            &new_journalist_msg_key_pair.public_key().clone(),
            &journalist_id_key_pair_2, // the new id key, not the key which signed the msg key!
            stack.now(),
        )
        .await
        .expect("posting msg key should succeed");

    // Assert the new msg key is included in API response
    let keys =
        get_and_verify_public_keys(stack.api_client_uncached(), &anchor_org_pks, stack.now())
            .await
            .keys;
    assert_eq!(keys.journalist_id_pk_iter().count(), 3);
    assert_eq!(keys.journalist_msg_pk_iter().count(), 3);

    // Assert that the new msg key is in the hierarchy under the first id key
    let id_key_1_msg_pks =
        keys.journalist_msg_pk_iter_for_id_pk(&journalist_id_key_pair_1.public_key());
    assert_eq!(
        id_key_1_msg_pks
            .max_by_key(|pk| pk.not_valid_after)
            .unwrap(),
        new_journalist_msg_key_pair.public_key()
    );
    // and there are no msg keys under id key 2
    let id_key_2_msg_pks =
        keys.journalist_msg_pk_iter_for_id_pk(&journalist_id_key_pair_2.public_key());
    assert_eq!(id_key_2_msg_pks.collect_vec().len(), 0);
}
