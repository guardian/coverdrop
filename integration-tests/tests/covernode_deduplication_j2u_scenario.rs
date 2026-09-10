use client::commands::{
    journalist::messages::send_journalist_to_user_cover_message,
    user::dead_drops::load_user_dead_drop_messages,
};
use common::protocol::journalist::encrypt_real_message_from_journalist_to_user_via_covernode;
use common::FixedSizeMessageText;
use integration_tests::{
    api_wrappers::{get_and_verify_public_keys, get_user_dead_drops},
    dev_j2u_mixing_config,
    stack::{CoverDropStack, StackProfile},
};
use std::time::Duration;

static JOURNALIST_MESSAGE: &str = "This is the journalist's reply to the user";

/// This test verifies that duplicate J2U messages are deduplicated in CoverNode.
#[tokio::test]
#[allow(clippy::await_holding_refcell_ref)]
async fn covernode_j2u_deduplication_scenario() {
    integration_tests::utils::init_logger();

    let stack = CoverDropStack::new(StackProfile::CoverDropOnly).await;

    let anchor_org_pks = stack.keys().anchor_org_pks();

    let keys_and_profiles =
        get_and_verify_public_keys(stack.api_client_cached(), &anchor_org_pks, stack.now()).await;

    //
    // Journalist replies to a user, publishing directly to Kinesis rather than via the API.
    // This avoids the API deduplication and checks the CoverNode deduplication.
    //
    {
        let journalist_vault = stack.load_static_journalist_vault().await;
        let user_mailbox = stack.mailboxes().user();

        let journalist_msg_key_pair = journalist_vault
            .latest_msg_key_pair(stack.now())
            .await
            .expect("Get latest journalist messaging key pair")
            .expect("No messaging key in journalist vault");

        let message = FixedSizeMessageText::new(JOURNALIST_MESSAGE).unwrap();

        let encrypted_outer_msg = encrypt_real_message_from_journalist_to_user_via_covernode(
            &keys_and_profiles.keys,
            &user_mailbox.user_key_pair().public_key(),
            &journalist_msg_key_pair,
            &message,
        )
        .expect("Encrypt real message from journalist to user");

        // Send many duplicates

        for _ in 0..5 {
            stack
                .kinesis_client()
                .encode_and_put_journalist_message(encrypted_outer_msg.clone())
                .await
                .expect("Send J2U message");
        }

        // trigger j2u dead drop
        for _ in 0..(dev_j2u_mixing_config().threshold_max - 5) {
            send_journalist_to_user_cover_message(stack.kinesis_client(), &keys_and_profiles.keys)
                .await
                .expect("Send journalist cover message");
        }

        tokio::time::sleep(Duration::from_secs(5)).await;
    }

    //
    // User reads dead drops and finds the journalist's reply
    //
    {
        let mut user_mailbox = stack.mailboxes().user();

        let dead_drop_list =
            get_user_dead_drops(stack.api_client_cached(), user_mailbox.max_dead_drop_id()).await;

        assert_eq!(dead_drop_list.len(), 1, "Expected exactly one dead drop");

        load_user_dead_drop_messages(
            &dead_drop_list,
            &keys_and_profiles.keys,
            &mut user_mailbox,
            stack.now(),
        )
        .expect("Save journalist's message to user mailbox");

        let messages = user_mailbox.messages().iter().collect::<Vec<_>>();

        assert_eq!(
            messages.len(),
            1,
            "Expected exactly 1 J2U message, but got {}",
            messages.len()
        );

        let message = messages[0].message.to_string().expect("Decode J2U message");
        assert_eq!(message, JOURNALIST_MESSAGE);
    }
}
