use std::time::Duration;

use common::{
    protocol::constants::{
        COVERNODE_PROVISIONING_KEY_ROTATE_AFTER_SECONDS,
        COVERNODE_PROVISIONING_KEY_VALID_DURATION_SECONDS, DAY_IN_SECONDS,
        ORGANIZATION_KEY_VALID_DURATION_SECONDS,
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

    // 2. Org pk still there, provisioing keys gone
    stack
        .time_travel(
            stack.now()
                + Duration::from_secs(COVERNODE_PROVISIONING_KEY_VALID_DURATION_SECONDS as u64),
        )
        .await;

    trigger_expired_key_deletion_identity_api(stack.identity_api_task_api_client()).await;

    let public_keys = stack
        .identity_api_client()
        .get_public_keys()
        .await
        .expect("Get public keys");

    assert!(!public_keys.anchor_org_pks.is_empty());
    assert!(public_keys.covernode_provisioning_pk.is_none());
    assert!(public_keys.journalist_provisioning_pk.is_none());

    // 3. All keys expired
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
