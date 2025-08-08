use std::{thread, time::Duration};

use client::commands::{
    journalist::{
        dead_drops::load_journalist_dead_drop_messages,
        messages::{send_journalist_to_user_cover_message, send_journalist_to_user_real_message},
    },
    user::{
        dead_drops::load_user_dead_drop_messages, messages::send_user_to_journalist_real_message,
    },
};
use integration_tests::{
    api_wrappers::{get_and_verify_public_keys, get_journalist_dead_drops, get_user_dead_drops},
    dev_j2u_mixing_config, dev_u2j_mixing_config, save_test_vector,
    utils::send_user_to_journalist_cover_messages,
    CoverDropStack,
};
use journalist_vault::VaultMessage;

static USER_MESSAGE: &str = "This is a test message from the user to the journalist";
static JOURNALIST_MESSAGE: &str = "This is a test message from the journalist to the user";
static JOURNALIST2_MESSAGE: &str = "This is a test message from the journalist 2 to the user";

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
async fn multiple_journalists_messaging_scenario() {
    pretty_env_logger::try_init().unwrap();

    let stack = CoverDropStack::builder()
        .with_additional_journalists(1)
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

        send_user_to_journalist_real_message(
            stack.messaging_client(),
            &mut user_mailbox,
            &keys_and_profiles.keys,
            &journalist_vault.journalist_id().await.unwrap(),
            USER_MESSAGE,
        )
        .await
        .expect("Send user real message");

        send_user_to_journalist_cover_messages(
            stack.messaging_client(),
            &keys_and_profiles.keys,
            dev_u2j_mixing_config().threshold_max - 1,
        )
        .await;

        thread::sleep(Duration::from_secs(5));
    }

    save_test_vector!("user_sent_message_and_processed", &stack);

    //
    // Journalist read dead drops
    //

    {
        let journalist_vault = stack.load_static_journalist_vault().await;

        let dead_drop_list = get_journalist_dead_drops(
            stack.api_client_cached(),
            journalist_vault.max_dead_drop_id().await.unwrap(),
        )
        .await;

        load_journalist_dead_drop_messages(
            dead_drop_list,
            &keys_and_profiles.keys,
            &journalist_vault,
            stack.now(),
        )
        .await
        .expect("Save journalist's messages to vault");

        assert_eq!(journalist_vault.messages().await.unwrap().len(), 1);

        let message = match &journalist_vault.messages().await.unwrap()[0] {
            VaultMessage::U2J(m) => m.message.clone(),
            _ => panic!("Expected U2J message"),
        };
        let message = message.to_string();

        assert_eq!(&message, USER_MESSAGE);
    }

    //
    // Journalist 1 replies
    //

    {
        let journalist_vault = stack.load_static_journalist_vault().await;

        let messages = journalist_vault.messages().await.unwrap();

        let user_pk = match &messages[0] {
            VaultMessage::U2J(m) => m.user_pk.clone(),
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

    save_test_vector!("journalist_1_replied_and_processed", &stack);

    //
    // Journalists 2 replies
    //

    {
        // This simulates Journalist 1 handing a user key directly to Journalist 2
        // This is only for generating test vectors, and will change when we have a
        // real handover protocol

        let user_pk = {
            let journalist_vault = stack.load_static_journalist_vault().await;

            let messages = journalist_vault.messages().await.unwrap();
            match &messages[0] {
                VaultMessage::U2J(m) => m.user_pk.clone(),
                _ => panic!("Expected U2J message"),
            }
        };

        let journalist_vault = stack.load_additional_journalist_vault(1).await;

        send_journalist_to_user_real_message(
            stack.kinesis_client(),
            &keys_and_profiles.keys,
            &journalist_vault,
            &user_pk,
            JOURNALIST2_MESSAGE,
            stack.now(),
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

    save_test_vector!("journalist_2_replied_and_processed", &stack);

    //
    // User read dead drops
    //

    {
        let mut user_mailbox = stack.mailboxes().user();

        let dead_drop_list =
            get_user_dead_drops(stack.api_client_cached(), user_mailbox.max_dead_drop_id()).await;

        assert_eq!(dead_drop_list.len(), 2);

        load_user_dead_drop_messages(
            &dead_drop_list,
            &keys_and_profiles.keys,
            &mut user_mailbox,
            stack.now(),
        )
        .expect("Save users's messages to mailbox");

        let messages = user_mailbox.messages().iter().collect::<Vec<_>>();

        // User has 3 messages, their own and the 2 responses from the journalists
        assert_eq!(messages.len(), 3);

        let message = messages[0].message.to_string().expect("Decode PCS");
        assert_eq!(&message, USER_MESSAGE);

        let message = messages[1].message.to_string().expect("Decode PCS");
        assert_eq!(&message, JOURNALIST_MESSAGE);

        let message = messages[2].message.to_string().expect("Decode PCS");
        assert_eq!(&message, JOURNALIST2_MESSAGE);
    }
    assert!(!stack.do_secrets_exist_in_stack().await);
}
