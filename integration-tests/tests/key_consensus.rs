use api::cache_control::PUBLIC_KEYS_TTL;
use chrono::{DateTime, Utc};
use client::commands::user::messages::send_user_to_journalist_real_message;
use common::protocol::constants::COVERNODE_MSG_KEY_ROTATE_AFTER;
use coverdrop_service::JournalistCoverDropService;
use integration_tests::{
    api_wrappers::{
        get_and_verify_public_keys, get_journalist_to_user_dead_drops,
        get_user_to_journalist_dead_drops,
    },
    dev_u2j_mixing_config, save_test_vector,
    stack::{CoverDropStack, StackProfile},
    utils::send_user_to_journalist_cover_messages,
};
use std::time::Duration;

const INITIAL_USER_MESSAGE: &str = "This is the initial test message from user to the journalist";
const SLEEP_DURATION: Duration = Duration::from_secs(5);

/// Tests the following key consensus issue related to cached responses from the API:
/// - A covernode rotates its messaging keys, creating key 2, and sends the public key to the API
/// - The covernode encrypts C2J messages and publishes a dead drop using key 2
/// - The journalist receives the dead drop and pulls the key hierarchy from the API, receiving
///   a cached response from before key 2 was created. Its initial attempt to
///   decrypt the C2J message is aborted, but a subsequent attempt receives the
///   new key and successfully decrypts the message.
#[tokio::test]
#[allow(clippy::await_holding_refcell_ref)]
async fn key_consensus() -> anyhow::Result<()> {
    integration_tests::utils::init_logger();

    let mut stack = CoverDropStack::builder(StackProfile::CoverDropOnly)
        .with_delete_old_dead_drops_poll_duration(SLEEP_DURATION)
        .with_varnish_api_cache(true)
        .build()
        .await;

    let anchor_org_pks = stack.keys().anchor_org_pks();
    let initial_keys_and_profiles =
        get_and_verify_public_keys(stack.api_client_cached(), &anchor_org_pks, stack.now()).await;

    //
    // Confirm clean initial state
    //
    {
        let user_dead_drops = get_journalist_to_user_dead_drops(
            stack.api_client_cached(),
            DateTime::<Utc>::UNIX_EPOCH,
        )
        .await;
        let journalist_dead_drops = get_user_to_journalist_dead_drops(
            stack.api_client_cached(),
            DateTime::<Utc>::UNIX_EPOCH,
        )
        .await;
        assert!(user_dead_drops.is_empty());
        assert!(journalist_dead_drops.is_empty());
    }

    save_test_vector!("initial_state", &stack);

    let initial_epoch = initial_keys_and_profiles.max_epoch;

    let journalist_id = {
        let journalist_vault = stack.load_static_journalist_vault().await;

        let journalist_id = journalist_vault
            .journalist_id()
            .await
            .expect("Get the journalist ID");

        //
        // Confirm clean initial state
        //
        {
            let user_dead_drops = get_journalist_to_user_dead_drops(
                stack.api_client_cached(),
                DateTime::<Utc>::UNIX_EPOCH,
            )
            .await;
            let journalist_dead_drops = get_user_to_journalist_dead_drops(
                stack.api_client_cached(),
                DateTime::<Utc>::UNIX_EPOCH,
            )
            .await;
            assert!(user_dead_drops.is_empty());
            assert!(journalist_dead_drops.is_empty());
        }

        save_test_vector!("initial_state", &stack);

        let initial_covernode_msg_pks_count = initial_keys_and_profiles
            .keys
            .covernode_msg_pk_iter()
            .count();

        assert!(initial_covernode_msg_pks_count == 1);

        journalist_id
    };

    // make covernode rotate messaging keys
    let after_covernode_key_rotation = stack.now() + COVERNODE_MSG_KEY_ROTATE_AFTER;

    stack.time_travel(after_covernode_key_rotation).await;

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    stack
        .time_travel(stack.now() + chrono::Duration::minutes(1))
        .await;

    // assert that uncached api response has new covernode msg pk
    let new_keys_and_profiles =
        get_and_verify_public_keys(stack.api_client_uncached(), &anchor_org_pks, stack.now()).await;

    assert!(new_keys_and_profiles.max_epoch > initial_epoch);
    let new_covernode_msg_pks_count = new_keys_and_profiles.keys.covernode_msg_pk_iter().count();
    assert_eq!(new_covernode_msg_pks_count, 2);

    // assert that the cached api response is equal to the initial one
    let cached_keys_and_profiles =
        get_and_verify_public_keys(stack.api_client_cached(), &anchor_org_pks, stack.now()).await;

    assert!(itertools::equal(
        cached_keys_and_profiles.keys.covernode_msg_pk_iter(),
        initial_keys_and_profiles.keys.covernode_msg_pk_iter()
    ));

    //
    // set up mailboxes
    //

    let mut user_mailbox = stack.mailboxes().user();

    //
    // User sends messages to journalist
    //
    send_user_to_journalist_real_message(
        stack.messaging_client(),
        &mut user_mailbox,
        &new_keys_and_profiles.keys,
        &journalist_id,
        INITIAL_USER_MESSAGE,
    )
    .await
    .expect("Send user real message");

    send_user_to_journalist_cover_messages(
        stack.messaging_client(),
        &new_keys_and_profiles.keys,
        dev_u2j_mixing_config().threshold_max - 1,
    )
    .await;

    tokio::time::sleep(SLEEP_DURATION).await;

    //
    // Wait for the covernode to publish the dead drop, which is encrypted with the covernode
    // messaging key that is not yet visible in the cached key hierarchy.
    //
    let dead_drop_published = {
        let mut published = false;
        for _ in 0..PUBLIC_KEYS_TTL.num_seconds() {
            let dead_drops = get_user_to_journalist_dead_drops(
                stack.api_client_uncached(),
                DateTime::<Utc>::UNIX_EPOCH,
            )
            .await;

            if !dead_drops.dead_drops.is_empty() {
                published = true;
                break;
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        published
    };

    assert!(dead_drop_published, "covernode published no u2j dead drop");

    //
    // Journalist pulls the dead drop. Dead drops are pulled from the uncached API so that the only
    // stale data in play is the key hierarchy.
    //
    let journalist_vault = stack.load_static_journalist_vault().await;
    let coverdrop_service =
        JournalistCoverDropService::new(stack.api_client_uncached(), &journalist_vault);

    //
    // The cached key hierarchy does not contain the covernode messaging key the dead drop was
    // encrypted with, so the journalist must abort without advancing its dead drop cursor,
    // otherwise the message would be skipped once the new key becomes visible.
    //
    let stale_keys_and_profiles =
        get_and_verify_public_keys(stack.api_client_cached(), &anchor_org_pks, stack.now()).await;

    assert_eq!(
        stale_keys_and_profiles.max_epoch, initial_epoch,
        "cached key hierarchy refreshed before the stale key scenario could be exercised"
    );

    let stale_messages = coverdrop_service
        .pull_and_decrypt_dead_drops(&stale_keys_and_profiles, None::<fn(usize)>, stack.now())
        .await
        .expect("Pull and decrypt dead drops with a stale key hierarchy");

    assert!(stale_messages.is_empty());
    assert_eq!(
        journalist_vault
            .max_dead_drop_created_at()
            .await
            .expect("Get max dead drop created at"),
        DateTime::<Utc>::UNIX_EPOCH
    );

    //
    // Once the cached key hierarchy catches up the same call decrypts the message.
    //
    let success: bool = {
        let mut success = false;
        // Retry for the duration of the TTL, checking once per second
        for i in 0..PUBLIC_KEYS_TTL.num_seconds() {
            let keys_and_profiles =
                get_and_verify_public_keys(stack.api_client_cached(), &anchor_org_pks, stack.now())
                    .await;

            let messages = coverdrop_service
                .pull_and_decrypt_dead_drops(&keys_and_profiles, None::<fn(usize)>, stack.now())
                .await
                .expect("Pull and decrypt dead drops");

            let found = messages.iter().any(|message| {
                message
                    .u2j_message
                    .message
                    .to_string()
                    .is_ok_and(|message| message.contains(INITIAL_USER_MESSAGE))
            });

            if found {
                assert!(keys_and_profiles.max_epoch > initial_epoch);
                success = true;
                break;
            } else {
                tracing::info!(
                    "Message not found, attempts remaining: {}",
                    PUBLIC_KEYS_TTL.num_seconds() - i
                );
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
        success
    };

    assert!(success);

    save_test_vector!("user_sent_message_and_processed", &stack);

    assert!(!stack.do_secrets_exist_in_stack().await);

    Ok(())
}
