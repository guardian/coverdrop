use std::time::Duration;

use common::{
    api::forms::PostCoverNodeProvisioningPublicKeyForm,
    protocol::{
        constants::{
            COVERNODE_ID_KEY_VALID_DURATION_SECONDS,
            COVERNODE_PROVISIONING_KEY_ROTATE_AFTER_SECONDS, DAY_IN_SECONDS,
            ORGANIZATION_KEY_VALID_DURATION_SECONDS,
        },
        keys::{
            generate_covernode_id_key_pair, generate_covernode_messaging_key_pair,
            generate_covernode_provisioning_key_pair,
        },
    },
    task::RunnerMode,
};

use integration_tests::{
    api_wrappers::{
        get_and_verify_public_keys, trigger_all_init_tasks_covernode,
        trigger_expired_key_deletion_covernode, trigger_expired_key_deletion_identity_api,
        trigger_key_rotation_covernode,
    },
    save_test_vector, CoverDropStack,
};
use itertools::Itertools;

/// This test rotates keys of of the CoverNode and then verifies that the keys are rotated.
#[tokio::test]
async fn covernode_key_rotations() {
    pretty_env_logger::try_init().unwrap();

    let mut stack = CoverDropStack::builder()
        .with_identity_api_task_runner_mode(RunnerMode::ManuallyTriggered)
        .with_covernode_task_runner_mode(RunnerMode::ManuallyTriggered)
        .build()
        .await;

    let anchor_org_pks = stack.keys().anchor_org_pks();

    // Since we are fully manually triggering the tasks, we need to trigger the initial tasks
    trigger_all_init_tasks_covernode(stack.covernode_task_api_client()).await;

    // We first capture the initial state as is
    let keys =
        get_and_verify_public_keys(stack.api_client_uncached(), &anchor_org_pks, stack.now())
            .await
            .keys;

    assert_eq!(keys.covernode_id_pk_iter().count(), 1);
    assert_eq!(keys.covernode_msg_pk_iter().count(), 1);

    save_test_vector!("initial", &stack);

    // We move one week and a bit into the future. Once triggered, the CoverNode should rotate its
    // messaging keys: `COVERNODE_MSG_KEY_ROTATE_AFTER_SECONDS: i64 = 7 * DAY_IN_SECONDS`
    stack
        .time_travel(stack.now() + Duration::from_secs(8 * DAY_IN_SECONDS as u64))
        .await;

    trigger_key_rotation_covernode(stack.covernode_task_api_client()).await;

    let keys =
        get_and_verify_public_keys(stack.api_client_uncached(), &anchor_org_pks, stack.now())
            .await
            .keys;

    assert_eq!(keys.covernode_id_pk_iter().count(), 1);
    assert_eq!(keys.covernode_msg_pk_iter().count(), 2);

    save_test_vector!("covernode_msg_rotated_1", &stack);

    // We remember the current msg pks so that we can compare them in the next step
    let msg_pks_week_1 = keys.covernode_msg_pk_iter().collect::<Vec<_>>();

    // We move another week and a bit into the future. Once triggered, the CoverNode should rotate
    // its messaging keys again and also its identity keys:
    // `COVERNODE_ID_KEY_ROTATE_AFTER_SECONDS: i64 = 2 * WEEK_IN_SECONDS`.
    stack
        .time_travel(stack.now() + Duration::from_secs(8 * DAY_IN_SECONDS as u64))
        .await;

    trigger_key_rotation_covernode(stack.covernode_task_api_client()).await;

    let keys =
        get_and_verify_public_keys(stack.api_client_uncached(), &anchor_org_pks, stack.now())
            .await
            .keys;

    // The very first messaging key will have expired and no longer show up in our public keys list.
    assert_eq!(keys.covernode_id_pk_iter().count(), 2);
    assert_eq!(keys.covernode_msg_pk_iter().count(), 2);

    // The soonest-to-expire key from week 1 is no longer in the list
    let oldest_msg_pk_week_1 = msg_pks_week_1
        .iter()
        .min_by_key(|x| x.1.not_valid_after)
        .unwrap();

    assert!(!keys.covernode_msg_pk_iter().contains(oldest_msg_pk_week_1));

    // The newest key was not in the list before
    let newest_msg_pk = keys
        .covernode_msg_pk_iter()
        .max_by_key(|x| x.1.not_valid_after)
        .unwrap();

    assert!(!msg_pks_week_1.contains(&newest_msg_pk));

    // Check the messaging key has been deleted from the database
    trigger_expired_key_deletion_covernode(stack.covernode_task_api_client()).await;

    let covernode_db = stack.covernode_database();
    let msg_key_pairs = covernode_db
        .select_published_msg_key_pairs()
        .await
        .expect("Get msg key pairs");

    assert_eq!(msg_key_pairs.len(), 2);

    save_test_vector!("covernode_msg_rotated_2", &stack);

    // Check key expiry
    //
    // 1. Things should be valid right now

    trigger_expired_key_deletion_identity_api(stack.identity_api_task_api_client()).await;

    let public_keys = stack
        .identity_api_client()
        .get_public_keys()
        .await
        .expect("Get public keys");

    assert!(!public_keys.anchor_org_pks.is_empty());
    assert!(public_keys.covernode_provisioning_pk.is_some());
    assert!(public_keys.journalist_provisioning_pk.is_some());

    // 2. All keys expired
    stack
        .time_travel(
            stack.now() + Duration::from_secs(ORGANIZATION_KEY_VALID_DURATION_SECONDS as u64),
        )
        .await;

    trigger_expired_key_deletion_identity_api(stack.identity_api_task_api_client()).await;

    let public_keys = stack
        .identity_api_client()
        .get_public_keys()
        .await
        .expect("Get public keys");

    assert!(public_keys.anchor_org_pks.is_empty());
    assert!(public_keys.covernode_provisioning_pk.is_none());
    assert!(public_keys.journalist_provisioning_pk.is_none());
}

/// This test simulates a very rare scenario where the covernode creates a candidate id key but fails to publish it before a
/// successful provisioning key rotation takes place. The API should accept the candidate id key signed by the old provisioning key.
#[tokio::test]
async fn concurrent_covernode_id_and_provisioning_key_rotations() {
    let mut stack = CoverDropStack::builder()
        .with_covernode_task_runner_mode(RunnerMode::ManuallyTriggered)
        .build()
        .await;

    let anchor_org_pks = stack.keys().anchor_org_pks();

    // Trigger the initial tasks
    trigger_all_init_tasks_covernode(stack.covernode_task_api_client()).await;

    // Assert initial keys created
    let keys =
        get_and_verify_public_keys(stack.api_client_uncached(), &anchor_org_pks, stack.now())
            .await
            .keys;
    assert_eq!(keys.covernode_provisioning_pk_iter().count(), 1);
    assert_eq!(keys.covernode_id_pk_iter().count(), 1);

    // time travel to avoid provisioning key being rejected by api
    stack
        .time_travel(
            stack.now()
                + Duration::from_secs(
                    COVERNODE_PROVISIONING_KEY_ROTATE_AFTER_SECONDS
                        .try_into()
                        .unwrap(),
                ),
        )
        .await;

    // Create a new covernode id key signed with the current covernode provisioning key
    let covernode_provisioning_key_pair_1 = &stack.keys().covernode_provisioning_key_pair;
    let new_covernode_id_key_pair =
        generate_covernode_id_key_pair(covernode_provisioning_key_pair_1, stack.now());

    // Rotate covernode provisioning key
    let covernode_provisioning_key_pair_2 =
        generate_covernode_provisioning_key_pair(&stack.keys().org_key_pair, stack.now());
    let covernode_provisioning_pk_form = PostCoverNodeProvisioningPublicKeyForm::new(
        covernode_provisioning_key_pair_2.to_untrusted().public_key,
        &stack.keys().org_key_pair,
        stack.now(),
    )
    .expect("creating provisioning pk form should succeed");
    stack
        .api_client_cached()
        .post_covernode_provisioning_pk(covernode_provisioning_pk_form)
        .await
        .expect("posted new provisioning key");

    // Assert new provisioning key included in API response
    let keys =
        get_and_verify_public_keys(stack.api_client_uncached(), &anchor_org_pks, stack.now())
            .await
            .keys;
    assert_eq!(keys.covernode_provisioning_pk_iter().count(), 2);
    assert_eq!(keys.covernode_id_pk_iter().count(), 0); // (original id key expired)

    // Post the new id key signed with the old provisioning key to the API
    stack
        .api_client_uncached()
        .post_covernode_id_pk(
            stack.covernode_id(),
            &new_covernode_id_key_pair.public_key().clone(),
            &covernode_provisioning_key_pair_2, // the new provisioning key, not the key which signed the id key!
            stack.now(),
        )
        .await
        .expect("posting id key should succeed");

    // Assert the new id key is included in API response
    let keys =
        get_and_verify_public_keys(stack.api_client_uncached(), &anchor_org_pks, stack.now())
            .await
            .keys;
    assert_eq!(keys.covernode_provisioning_pk_iter().count(), 2);
    assert_eq!(keys.covernode_id_pk_iter().count(), 1);

    // Assert that the new id key is in the hierarchy under the first provisioning key
    let provisioning_key_1_id_pks = keys
        .covernode_id_pk_iter_for_provisioning_pk(&covernode_provisioning_key_pair_1.public_key());
    assert_eq!(
        provisioning_key_1_id_pks
            .max_by_key(|pk| pk.not_valid_after)
            .unwrap(),
        new_covernode_id_key_pair.public_key()
    );
    // and there are no id keys under provisioning key 2
    let provisioning_key_2_id_pks = keys
        .covernode_id_pk_iter_for_provisioning_pk(&covernode_provisioning_key_pair_2.public_key());
    assert_eq!(provisioning_key_2_id_pks.collect_vec().len(), 0);
}

/// This test simulates a scenario where the covernode creates a candidate msg key but fails to publish it before a
/// successful identity key rotation. The API should accept the candidate msg key signed by the old identity key.
#[tokio::test]
async fn concurrent_covernode_msg_and_id_key_rotations() {
    let mut stack = CoverDropStack::builder()
        .with_identity_api_task_runner_mode(RunnerMode::ManuallyTriggered)
        .with_covernode_task_runner_mode(RunnerMode::ManuallyTriggered)
        .build()
        .await;

    let anchor_org_pks = stack.keys().anchor_org_pks();

    // Trigger the initial tasks
    trigger_all_init_tasks_covernode(stack.covernode_task_api_client()).await;

    // Assert initial keys created
    let keys =
        get_and_verify_public_keys(stack.api_client_uncached(), &anchor_org_pks, stack.now())
            .await
            .keys;
    assert_eq!(keys.covernode_id_pk_iter().count(), 1);
    assert_eq!(keys.covernode_msg_pk_iter().count(), 1);

    stack
        .time_travel(stack.now() + Duration::from_nanos(1))
        .await;

    // Create a new covernode msg key signed with the current covernode id key
    let covernode_id_key_pair_1 = &stack.keys().covernode_id_key_pair;
    let new_covernode_msg_key_pair =
        generate_covernode_messaging_key_pair(covernode_id_key_pair_1, stack.now());

    // Rotate covernode id key
    let covernode_id_key_pair_2 =
        generate_covernode_id_key_pair(&stack.keys().covernode_provisioning_key_pair, stack.now());
    stack
        .api_client_cached()
        .post_covernode_id_pk(
            &stack.covernode_id(),
            &covernode_id_key_pair_2.public_key().clone(),
            &stack.keys().covernode_provisioning_key_pair,
            stack.now(),
        )
        .await
        .expect("posted new id key");

    // Assert new id key included in API response
    let keys =
        get_and_verify_public_keys(stack.api_client_uncached(), &anchor_org_pks, stack.now())
            .await
            .keys;
    assert_eq!(keys.covernode_id_pk_iter().count(), 2);
    assert_eq!(keys.covernode_msg_pk_iter().count(), 1);

    // Post the new msg key signed with the old id key to the API
    stack
        .api_client_uncached()
        .post_covernode_msg_pk(
            &new_covernode_msg_key_pair.public_key().clone(),
            &covernode_id_key_pair_2, // the new id key, not the key which signed the msg key!
            stack.now(),
        )
        .await
        .expect("posting msg key should succeed");

    // Assert the new msg key is included in API response
    let keys =
        get_and_verify_public_keys(stack.api_client_uncached(), &anchor_org_pks, stack.now())
            .await
            .keys;
    assert_eq!(keys.covernode_id_pk_iter().count(), 2);
    assert_eq!(keys.covernode_msg_pk_iter().count(), 2);

    // Assert that the new msg key is in the hierarchy under the first id key
    let id_key_1_msg_pks =
        keys.covernode_msg_pk_iter_for_id_pk(&covernode_id_key_pair_1.public_key());
    assert_eq!(
        id_key_1_msg_pks
            .max_by_key(|pk| pk.not_valid_after)
            .unwrap(),
        new_covernode_msg_key_pair.public_key()
    );
    // and there are no msg keys under id key 2
    let id_key_2_msg_pks =
        keys.covernode_msg_pk_iter_for_id_pk(&covernode_id_key_pair_2.public_key());
    assert_eq!(id_key_2_msg_pks.collect_vec().len(), 0);
}

/// This test simulates a scenario where the covernode creates a new identity key near the end of the parent provisioning key's expiry.
/// This should lead to a truncated identity key lifetime, but not trigger a id key rotation.
/// Similarly, a new messaging key is created near the end of the identity key's expiry,
/// leading to a truncated messaging key lifetime, but not triggering a msg key rotation
#[tokio::test]
async fn late_covernode_msg_and_id_key_rotations() {
    let mut stack = CoverDropStack::builder()
        .with_identity_api_task_runner_mode(RunnerMode::ManuallyTriggered)
        .with_covernode_task_runner_mode(RunnerMode::ManuallyTriggered)
        .build()
        .await;

    let anchor_org_pks = stack.keys().anchor_org_pks();

    // Trigger the initial tasks
    trigger_all_init_tasks_covernode(stack.covernode_task_api_client()).await;

    // Assert initial keys created
    let keys =
        get_and_verify_public_keys(stack.api_client_uncached(), &anchor_org_pks, stack.now())
            .await
            .keys;
    assert_eq!(keys.covernode_id_pk_iter().count(), 1);
    assert_eq!(keys.covernode_msg_pk_iter().count(), 1);

    async fn id_keys_expiry_match_provisioning(stack: &CoverDropStack) -> bool {
        let keys = get_and_verify_public_keys(
            stack.api_client_uncached(),
            &stack.keys().anchor_org_pks(),
            stack.now(),
        )
        .await
        .keys;

        keys.latest_covernode_id_pk(stack.covernode_id())
            .unwrap()
            .not_valid_after
            == keys
                .latest_covernode_provisioning_pk()
                .unwrap()
                .not_valid_after
    }
    // Step time forward to near the expiry of the provisioning key
    // This will get us into a state where the most recent id key will have a truncated lifetime
    while !id_keys_expiry_match_provisioning(&stack).await {
        stack
            .time_travel(
                stack.now()
                    + Duration::from_secs(
                        (COVERNODE_ID_KEY_VALID_DURATION_SECONDS - DAY_IN_SECONDS)
                            .try_into()
                            .unwrap(),
                    ),
            )
            .await;

        trigger_key_rotation_covernode(stack.covernode_task_api_client()).await;
    }

    let keys =
        get_and_verify_public_keys(stack.api_client_uncached(), &anchor_org_pks, stack.now())
            .await
            .keys;

    let latest_id_key = keys.latest_covernode_id_pk(stack.covernode_id()).unwrap();

    // Now we are in a state where the most recent id key will have a truncated lifetime
    // Trigger creation of a new id and msg key, this should not cause a rotation since we are within the allowed window
    trigger_key_rotation_covernode(stack.covernode_task_api_client()).await;

    let keys =
        get_and_verify_public_keys(stack.api_client_uncached(), &anchor_org_pks, stack.now())
            .await
            .keys;

    let new_latest_id_key = keys.latest_covernode_id_pk(stack.covernode_id()).unwrap();

    // Assert no new id key created
    assert_eq!(latest_id_key, new_latest_id_key);
}
