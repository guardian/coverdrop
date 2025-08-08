use std::{
    thread::{self, sleep},
    time::Duration,
};

use client::commands::{
    journalist::{
        dead_drops::load_journalist_dead_drop_messages,
        messages::{send_journalist_to_user_cover_message, send_journalist_to_user_real_message},
    },
    user::{
        dead_drops::load_user_dead_drop_messages,
        messages::{send_user_to_journalist_cover_message, send_user_to_journalist_real_message},
    },
};
use integration_tests::{
    api_wrappers::{get_and_verify_public_keys, get_journalist_dead_drops, get_user_dead_drops},
    dev_j2u_mixing_config, dev_u2j_mixing_config, save_test_vector, CoverDropStack,
};
use journalist_vault::VaultMessage;

static USER_MESSAGE: &str = "This is a test message from the user to the journalist";
static JOURNALIST_MESSAGE: &str = "This is a test message from the journalist to the user";
const DELETE_OLD_DEAD_DROPS_POLLING_PERIOD_SECONDS: u64 = 2;

/// This test is a high level integration test that tests the full messaging scenario.
///
/// It starts a full stack, generates a journalist and a user, and then sends messages
/// from the user to the journalist checking the dead drops for the message. It then does the
/// same in reverse, sending a message from the journalist to the user.
#[tokio::test]
// This is a sensible lint that can prevent you from accidentally panicking in production code
// but we're using `RefCell`s in these tests for ergonomic improvements, namely allowing multiple
// mutable borrows of the mailboxes.
//
// If it turns out we do something unsafe that causes panics we can find some other way to have
// ergonomically written integration tests
#[allow(clippy::await_holding_refcell_ref)]
async fn messaging_scenario() {
    pretty_env_logger::try_init().unwrap();

    let mut stack = CoverDropStack::builder()
        .with_delete_old_dead_drops_poll_seconds(
            DELETE_OLD_DEAD_DROPS_POLLING_PERIOD_SECONDS
                .try_into()
                .unwrap(),
        )
        .build()
        .await;

    let anchor_org_pks = stack.keys().anchor_org_pks();

    let keys_and_profiles =
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

    //
    // User sends messages to journalist
    //

    {
        let journalist_vault = stack.load_static_journalist_vault().await;
        let mut user_mailbox = stack.mailboxes().user();

        let journalist_id = journalist_vault
            .journalist_id()
            .await
            .expect("Get the journalist ID");

        send_user_to_journalist_real_message(
            stack.messaging_client(),
            &mut user_mailbox,
            &keys_and_profiles.keys,
            &journalist_id,
            USER_MESSAGE,
        )
        .await
        .expect("Send user real message");

        for i in 0..(dev_u2j_mixing_config().threshold_max - 1) {
            // Halfway through sending U2J messages - double the number of kinesis shards
            if i == dev_u2j_mixing_config().threshold_max / 2 {
                stack.scale_kinesis().await;
            }

            send_user_to_journalist_cover_message(stack.messaging_client(), &keys_and_profiles.keys)
                .await
                .expect("Send use cover message")
        }

        thread::sleep(Duration::from_secs(5));
    }

    save_test_vector!("user_sent_message_and_processed", &stack);

    //
    // Journalist read dead drops
    //

    {
        let journalist_vault = stack.load_static_journalist_vault().await;

        let max_dead_drop_id = journalist_vault
            .max_dead_drop_id()
            .await
            .expect("Get max dead drop id");

        let dead_drop_list =
            get_journalist_dead_drops(stack.api_client_cached(), max_dead_drop_id).await;

        load_journalist_dead_drop_messages(
            dead_drop_list,
            &keys_and_profiles.keys,
            &journalist_vault,
            stack.now(),
        )
        .await
        .expect("Save journalist's messages to vault");

        let messages = journalist_vault
            .messages()
            .await
            .expect("Get journalist messages");

        assert_eq!(messages.len(), 1);

        let message = match messages[0] {
            VaultMessage::U2J(ref m) => &m.message,
            _ => panic!("Expected U2J message"),
        };

        assert_eq!(message, USER_MESSAGE);
    }

    //
    // Journalists replies
    //

    {
        let journalist_vault = stack.load_static_journalist_vault().await;

        let messages = journalist_vault
            .messages()
            .await
            .expect("Get journalist messages");

        let user_pk = match messages[0] {
            VaultMessage::U2J(ref m) => m.user_pk.clone(),
            _ => panic!("Expected U2J message"),
        };

        // Slightly annoying `Copy` of now to avoid a borrow checker error
        let now = stack.now();
        send_journalist_to_user_real_message(
            stack.kinesis_client(),
            &keys_and_profiles.keys,
            &journalist_vault,
            &user_pk,
            JOURNALIST_MESSAGE,
            now,
        )
        .await
        .expect("Send message");

        for _ in 0..(dev_j2u_mixing_config().threshold_max - 1) {
            send_journalist_to_user_cover_message(stack.kinesis_client(), &keys_and_profiles.keys)
                .await
                .expect("Send journalist cover message")
        }

        thread::sleep(Duration::from_secs(5));
    }

    save_test_vector!("journalist_replied_and_processed", &stack);

    //
    // User read dead drops
    //
    {
        let mut user_mailbox = stack.mailboxes().user();

        let dead_drop_list =
            get_user_dead_drops(stack.api_client_cached(), user_mailbox.max_dead_drop_id()).await;

        assert_eq!(dead_drop_list.len(), 1);

        load_user_dead_drop_messages(
            &dead_drop_list,
            &keys_and_profiles.keys,
            &mut user_mailbox,
            stack.now(),
        )
        .expect("Save users's messages to mailbox");

        let messages = user_mailbox.messages().iter().collect::<Vec<_>>();

        // User has 2 messages, their own and the response for the journalist
        assert_eq!(messages.len(), 2);

        let message = messages[0].message.to_string().expect("Decode PCS");
        assert_eq!(&message, USER_MESSAGE);

        let message = messages[1].message.to_string().expect("Decode PCS");
        assert_eq!(&message, JOURNALIST_MESSAGE);
    }

    //
    // Ensure API deletes old dead drops - emulate a new user attempting to pull the dead drops from the beginning
    //
    {
        let vault = stack.load_static_journalist_vault().await;

        let dead_drop_list = get_user_dead_drops(stack.api_client_cached(), 0).await;
        assert_eq!(dead_drop_list.len(), 1);
        let first_dead_drop = dead_drop_list.dead_drops.first().unwrap();

        // First check we can time travel to 13 days after now, the dead drops should still be there
        let still_exists_time = stack.now() + chrono::Duration::days(13);
        stack.time_travel(still_exists_time).await;

        // Wait for the dead drop deletion poller to run
        sleep(Duration::from_secs(
            DELETE_OLD_DEAD_DROPS_POLLING_PERIOD_SECONDS + 1,
        ));

        // There seems to be some timing related issues where an extra dead drop is sometimes published
        // So we will assert that the original dead drop still exists.
        let dead_drop_list = get_user_dead_drops(stack.api_client_cached(), 0).await;
        assert!(
            dead_drop_list.dead_drops.contains(first_dead_drop),
            "Unexpected number of dead drops 13 days after publication"
        );

        // Messages have not yet been garbage collected from journalist vault
        vault.clean_up(stack.now()).await.expect("Clean up vault");
        assert!(vault.messages().await.unwrap().len() == 2);

        // Travel an extra day forward in time, there should be no dead drops now.
        let no_dead_drops_time = stack.now() + chrono::Duration::days(1);
        stack.time_travel(no_dead_drops_time).await;

        // Wait for the dead drop deletion poller to run
        sleep(Duration::from_secs(
            DELETE_OLD_DEAD_DROPS_POLLING_PERIOD_SECONDS + 1,
        ));

        // After 14 days the original dead drop should be deleted.
        let dead_drop_list_after_delete = get_user_dead_drops(stack.api_client_cached(), 0).await;
        assert!(
            !dead_drop_list_after_delete
                .dead_drops
                .contains(first_dead_drop),
            "failed to delete user dead drop after expiry time {dead_drop_list_after_delete:?}"
        );

        // Messages have been garbage collected from journalist vault
        vault.clean_up(stack.now()).await.expect("Clean up vault");
        assert!(vault.messages().await.unwrap().is_empty());

        // No secrets in logs
        assert!(!stack.do_secrets_exist_in_stack().await);
    }

    save_test_vector!("dead_drop_expired_and_no_longer_displayed", &stack);
}
