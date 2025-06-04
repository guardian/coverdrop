use std::time::Duration;

use client::commands::{
    journalist::messages::{
        send_journalist_to_user_cover_message, send_journalist_to_user_real_message,
    },
    user::dead_drops::load_user_dead_drop_messages,
};
use common::{
    api::models::messages::{
        user_to_journalist_message::UserToJournalistMessage,
        user_to_journalist_message_with_dead_drop_id::UserToJournalistMessageWithDeadDropId,
    },
    client::mailbox::user_mailbox::MAX_MAILBOX_MESSAGES,
    FixedSizeMessageText,
};
use integration_tests::{
    api_wrappers::{get_and_verify_public_keys, get_user_dead_drops},
    dev_u2j_mixing_config, CoverDropStack,
};
use rand::Rng;
use tokio::time;

const MESSAGES: [&str; 3] = ["Hello", "안녕하세요", "こんにちは"];

// This needs to be a reasonably high number since we want to cause the user mailbox to wrap
// At the moment a few values in this test are hard coded - once the threshold mixer becomes dynamic
// we will have to alter our approach, but as of 2023-04-26 the threshold mixer has static
// input and output sizes
const J2U_MESSAGES_TO_SEND: usize = 500;

enum RandomMessage {
    Real(String),
    Cover,
}

/// This test "fuzzes" (in a weak sense) the user and journalist mailboxes by
/// sending a large number of cover and real messages and then confirming that the final state
/// of the mailbox matches expectations.
#[tokio::test]
#[allow(clippy::await_holding_refcell_ref)] // See messaging scenario for explainer
async fn user_mailbox_filling_scenario() {
    pretty_env_logger::try_init().unwrap();

    let stack = CoverDropStack::new().await;

    let anchor_org_pks = stack.keys().anchor_org_pks();

    let keys_and_profiles =
        get_and_verify_public_keys(stack.api_client_cached(), &anchor_org_pks, stack.now()).await;

    let journalist_vault = stack.load_static_journalist_vault().await;
    let mut user_mailbox = stack.mailboxes().user();

    let messages = vec![UserToJournalistMessageWithDeadDropId {
        u2j_message: UserToJournalistMessage::new(
            FixedSizeMessageText::new("Seeding journalist vault with user to reply to").unwrap(),
            user_mailbox.user_key_pair().public_key(),
        ),
        dead_drop_id: 0,
    }];

    journalist_vault
        .add_messages_from_user_to_journalist_and_update_max_dead_drop_id(&messages, 0, stack.now())
        .await
        .expect("Add message to journalist vault");

    let user_pk = user_mailbox.user_key_pair().public_key();

    let mut rng = rand::thread_rng();

    //
    // Generate mailbox state
    //

    let mut message_queue = vec![];

    // Approximately 250 real messages.
    // This will emulate a user spilling their message capacity since user mailboxes
    // can only store 128 messages
    for i in 0..J2U_MESSAGES_TO_SEND {
        let is_real = rng.gen_bool(0.05);
        if is_real {
            let random_message_idx = rng.gen_range(0..MESSAGES.len());
            let message = format!("{:0>#6} - {}", i, MESSAGES[random_message_idx]);
            message_queue.push(RandomMessage::Real(message));
        } else {
            message_queue.push(RandomMessage::Cover);
        }
    }

    for _ in 0..(2 * dev_u2j_mixing_config().threshold_max) {
        message_queue.push(RandomMessage::Cover);
    }

    //
    // Send messages
    //

    for msg in &message_queue {
        match msg {
            RandomMessage::Real(text) => {
                send_journalist_to_user_real_message(
                    stack.kinesis_client(),
                    &keys_and_profiles.keys,
                    &journalist_vault,
                    user_pk,
                    text,
                    stack.now(),
                )
                .await
                .expect("Send message");
            }
            RandomMessage::Cover => send_journalist_to_user_cover_message(
                stack.kinesis_client(),
                &keys_and_profiles.keys,
            )
            .await
            .expect("Send journalist cover message"),
        }
    }

    //
    // Poll the dead drop end points until you get all the dead drops from the journalists
    // then verify the messages have all been sent correctly and are in the mailbox
    //

    let mut dead_drop_list = get_user_dead_drops(stack.api_client_cached(), 0).await;

    // Poll the dead drops every 5 seconds until we don't get any more
    let mut prev_dead_drop_len = 0;
    while dead_drop_list.len() != prev_dead_drop_len {
        prev_dead_drop_len = dead_drop_list.len();
        time::sleep(Duration::from_secs(2)).await;
        dead_drop_list = get_user_dead_drops(stack.api_client_cached(), 0).await;
    }

    println!("Sent {} messages", message_queue.len());
    println!("Dead Drops...");
    println!("  Got {} dead drops", dead_drop_list.len());

    let loaded_count = load_user_dead_drop_messages(
        &dead_drop_list,
        &keys_and_profiles.keys,
        &mut user_mailbox,
        stack.now(),
    )
    .expect("Save user's messages to mailbox");

    println!("Messages...");
    let expected_count = message_queue
        .iter()
        .filter(|m| matches!(m, RandomMessage::Real(_)))
        .count();

    println!("  Expected {expected_count}");
    println!("  Loaded {loaded_count}");
    println!("  Diff {}", expected_count - loaded_count);

    let mut expected_message_text = message_queue
        .iter()
        .rev()
        .filter_map(|m| match m {
            RandomMessage::Real(s) => Some(s),
            RandomMessage::Cover => None,
        })
        .take(MAX_MAILBOX_MESSAGES)
        .cloned()
        .collect::<Vec<String>>();
    expected_message_text.sort();

    let mut actual_message_text = user_mailbox
        .messages()
        .iter()
        .map(|msg| {
            msg.message
                .to_string()
                .expect("Convert message text to String")
        })
        .collect::<Vec<String>>();
    actual_message_text.sort();

    // Should be no difference between the actual messages in the mailbox and the
    // expected messages
    assert_eq!(expected_message_text, actual_message_text);

    assert!(!stack.do_secrets_exist_in_stack().await);
}
