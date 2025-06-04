use api::cache_control::PUBLIC_KEYS_TTL_IN_SECONDS;
use client::commands::user::{
    dead_drops::load_user_dead_drop_messages, messages::send_user_to_journalist_real_message,
};
use common::protocol::constants::COVERNODE_MSG_KEY_ROTATE_AFTER_SECONDS;
use integration_tests::{
    api_wrappers::{get_and_verify_public_keys, get_journalist_dead_drops, get_user_dead_drops},
    dev_u2j_mixing_config, save_test_vector,
    utils::send_user_to_journalist_cover_messages,
    CoverDropStack,
};
use std::{
    thread::{self},
    time::Duration,
};

const INITIAL_USER_MESSAGE: &str = "This is the initial test message from user to the journalist";
const SLEEP_DURATION: u64 = 5;

/// Tests the following key consensus issue related to cached responses from the API:
/// - A covernode rotates its messaging keys, creating key 2, and sends the public key to the API
/// - The covernode encrypts C2J messages and publishes a dead drop using key 2
/// - The signal bridge receives the dead drop and pulls the key hierarchy from the API, receiving
///   a cached response from before key 2 was created. Its initial attempt to
///   decrypt the C2J message is aborted, but a subsequent attempt receives the
///   new key and successfully decrypts the message.
#[tokio::test]
#[allow(clippy::await_holding_refcell_ref)]
async fn key_consensus() -> anyhow::Result<()> {
    pretty_env_logger::try_init().unwrap();

    let mut stack = CoverDropStack::builder()
        .with_delete_old_dead_drops_poll_seconds(2)
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
        let user_dead_drops = get_user_dead_drops(stack.api_client_cached(), 0).await;
        let journalist_dead_drops = get_journalist_dead_drops(stack.api_client_cached(), 0).await;
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
            let user_dead_drops = get_user_dead_drops(stack.api_client_cached(), 0).await;
            let journalist_dead_drops =
                get_journalist_dead_drops(stack.api_client_cached(), 0).await;
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
    let after_covernode_key_rotation = stack.now()
        + Duration::from_secs(COVERNODE_MSG_KEY_ROTATE_AFTER_SECONDS.try_into().unwrap());

    stack.time_travel(after_covernode_key_rotation).await;

    thread::sleep(Duration::from_secs(2));

    stack
        .time_travel(stack.now() + Duration::from_secs(60.try_into().unwrap()))
        .await;

    // assert that uncached api response has new covernode msg pk
    let new_keys_and_profiles =
        get_and_verify_public_keys(stack.api_client_uncached(), &anchor_org_pks, stack.now()).await;

    assert!(new_keys_and_profiles.max_epoch > initial_epoch);
    let new_covernode_msg_pks_count = new_keys_and_profiles.keys.covernode_msg_pk_iter().count();
    assert!(new_covernode_msg_pks_count == 2);

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

    thread::sleep(Duration::from_secs(SLEEP_DURATION));

    let success: bool = {
        let mut success = false;
        let num_attempts = PUBLIC_KEYS_TTL_IN_SECONDS;
        for i in 0..num_attempts {
            let keys_and_profiles = get_and_verify_public_keys(
                stack.api_client_uncached(),
                &anchor_org_pks,
                stack.now(),
            )
            .await;

            let dead_drop_list =
                get_user_dead_drops(stack.api_client_cached(), user_mailbox.max_dead_drop_id())
                    .await;

            load_user_dead_drop_messages(
                &dead_drop_list,
                &keys_and_profiles.keys,
                &mut user_mailbox,
                stack.now(),
            )
            .expect("Save users's messages to mailbox");

            let messages = user_mailbox
                .messages()
                .iter()
                .map(|mm| {
                    mm.message
                        .to_string()
                        .expect("read mailbox message to string")
                })
                .collect::<Vec<_>>();

            if messages.iter().any(|r| r.contains(INITIAL_USER_MESSAGE)) {
                success = true;
                break;
            } else {
                tracing::info!(
                    "Message not found, attempts remaining: {}",
                    num_attempts - i
                );
                thread::sleep(Duration::from_secs(1));
            }
        }
        success
    };

    assert!(success);

    save_test_vector!("user_sent_message_and_processed", &stack);

    assert!(!stack.do_secrets_exist_in_stack().await);

    Ok(())
}
