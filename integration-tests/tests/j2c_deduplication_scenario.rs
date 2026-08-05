use client::commands::user::dead_drops::load_user_dead_drop_messages;
use common::api::models::message_id::MessageId;
use coverdrop_service::JournalistCoverDropService;
use integration_tests::{
    api_wrappers::{get_and_verify_public_keys, get_user_dead_drops},
    dev_j2u_mixing_config,
    stack::{CoverDropStack, StackProfile},
};
use std::time::Duration;

static JOURNALIST_MESSAGE_1: &str = "This is the first message";
static JOURNALIST_MESSAGE_2: &str = "This is the duplicate message";

/// This test confirms that the API's J2C message deduplication logic works correctly.
///
/// A journalist enqueues and sends two messages with the same deduplication ID
/// via the `JournalistCoverDropService`. The API should deduplicate the second
/// send, so only one copy of the message arrives in the user's dead drops.
#[tokio::test]
#[allow(clippy::await_holding_refcell_ref)]
async fn j2c_deduplication_scenario() {
    let stack = CoverDropStack::new(StackProfile::CoverDropOnly).await;

    let anchor_org_pks = stack.keys().anchor_org_pks();

    let keys_and_profiles =
        get_and_verify_public_keys(stack.api_client_cached(), &anchor_org_pks, stack.now()).await;

    let journalist_vault = stack.load_static_journalist_vault().await;
    let mut user_mailbox = stack.mailboxes().user();
    let user_pk = user_mailbox.user_key_pair().public_key();

    let service = JournalistCoverDropService::new(stack.api_client_uncached(), &journalist_vault);

    // Use a fixed deduplication ID for both sends
    let deduplication_id = MessageId::new();

    // First send: enqueue and send the message
    service
        .enqueue_j2u_message(
            &keys_and_profiles,
            user_pk,
            JOURNALIST_MESSAGE_1,
            deduplication_id,
            stack.now(),
        )
        .await
        .expect("Enqueue first message");

    service
        .dequeue_and_send_j2u_message(&keys_and_profiles.keys, stack.now())
        .await
        .expect("First dequeue and send should succeed");

    // Second send: enqueue the same message with the same deduplication ID,
    // simulating a retry after a crash before the queue entry was deleted.
    service
        .enqueue_j2u_message(
            &keys_and_profiles,
            user_pk,
            JOURNALIST_MESSAGE_2,
            deduplication_id,
            stack.now(),
        )
        .await
        .expect("Enqueue duplicate message");

    service
        .dequeue_and_send_j2u_message(&keys_and_profiles.keys, stack.now())
        .await
        .expect("Second dequeue and send (duplicate) should also succeed without error");

    // Send cover messages to trigger the J2U mixer
    for _ in 0..(dev_j2u_mixing_config().threshold_max - 1) {
        service
            .dequeue_and_send_j2u_message(&keys_and_profiles.keys, stack.now())
            .await
            .expect("Send cover message via dequeue");
    }

    tokio::time::sleep(Duration::from_secs(5)).await;

    // Pull user dead drops and verify only one copy of the message arrived
    let dead_drop_list =
        get_user_dead_drops(stack.api_client_cached(), user_mailbox.max_dead_drop_id()).await;

    assert_eq!(dead_drop_list.len(), 1, "Expected exactly one dead drop");

    load_user_dead_drop_messages(
        &dead_drop_list,
        &keys_and_profiles.keys,
        &mut user_mailbox,
        stack.now(),
    )
    .expect("Load user dead drop messages");

    let messages = user_mailbox.messages().iter().collect::<Vec<_>>();

    assert_eq!(
        messages.len(),
        1,
        "Expected exactly one message, but got {}",
        messages.len()
    );

    let decrypted = messages[0].message.to_string().expect("Decode message");
    assert_eq!(&decrypted, JOURNALIST_MESSAGE_1);
}
