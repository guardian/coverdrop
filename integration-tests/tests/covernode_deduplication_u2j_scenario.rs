use client::commands::user::messages::send_user_to_journalist_cover_message;
use common::protocol::user::encrypt_real_message_from_user_to_journalist_via_covernode;
use common::FixedSizeMessageText;
use coverdrop_service::JournalistCoverDropService;
use integration_tests::{
    api_wrappers::get_and_verify_public_keys,
    dev_u2j_mixing_config,
    stack::{CoverDropStack, StackProfile},
};
use journalist_vault::VaultMessage;
use std::time::Duration;

static USER_MESSAGE: &str = "This is a message that should only appear once";

/// This test verifies that duplicate U2J messages are deduplicated in the CoverNode.
#[tokio::test]
#[allow(clippy::await_holding_refcell_ref)]
async fn covernode_u2j_deduplication_scenario() {
    integration_tests::utils::init_logger();

    let stack = CoverDropStack::new(StackProfile::CoverDropOnly).await;

    let anchor_org_pks = stack.keys().anchor_org_pks();

    let keys_and_profiles =
        get_and_verify_public_keys(stack.api_client_cached(), &anchor_org_pks, stack.now()).await;

    //
    // User sends the same real message five times, plus cover messages to trigger a dead drop
    //
    {
        let journalist_vault = stack.load_static_journalist_vault().await;
        let user_mailbox = stack.mailboxes().user();

        let journalist_id = journalist_vault
            .journalist_id()
            .await
            .expect("Get the journalist ID");

        let user_pk = user_mailbox.user_key_pair().public_key();

        let message = FixedSizeMessageText::new(USER_MESSAGE).unwrap();

        let encrypted_outer_msg = encrypt_real_message_from_user_to_journalist_via_covernode(
            &keys_and_profiles.keys,
            user_pk,
            &journalist_id,
            message,
        )
        .expect("Encrypt real message from user to journalist");

        // Send many duplicates

        for _ in 0..5 {
            stack
                .messaging_client()
                .post_user_message(encrypted_outer_msg.clone())
                .await
                .expect("Send U2J message");
        }

        // trigger u2j dead drop
        for _ in 0..(dev_u2j_mixing_config().threshold_max - 5) {
            send_user_to_journalist_cover_message(
                stack.messaging_client(),
                &keys_and_profiles.keys,
            )
            .await
            .expect("Send user cover message");
        }

        tokio::time::sleep(Duration::from_secs(5)).await;
    }

    //
    // Journalist fetches dead drops and decrypts and stores u2j messages
    //
    {
        let journalist_vault = stack.load_static_journalist_vault().await;

        let journalist_service =
            JournalistCoverDropService::new(stack.api_client_uncached(), &journalist_vault);

        let public_keys = get_and_verify_public_keys(
            stack.api_client_uncached(),
            &stack.keys().anchor_org_pks(),
            stack.now(),
        )
        .await;

        let pulled_and_decrypted_messages = journalist_service
            .pull_and_decrypt_dead_drops(&public_keys, None::<fn(usize)>, stack.now())
            .await
            .expect("Pull and decrypt dead drops");

        // Assert that only a single message is added to the dead drop
        assert_eq!(
            pulled_and_decrypted_messages.len(),
            1,
            "Expected exactly 1 U2J message, but got {}",
            pulled_and_decrypted_messages.len()
        );

        let u2j_message = &pulled_and_decrypted_messages[0].u2j_message;

        assert_eq!(
            u2j_message.message.to_string().expect("Unpack U2J message"),
            USER_MESSAGE
        );

        // Assert that only a single message is added to the journalist's vault
        let vault_messages = journalist_vault
            .messages()
            .await
            .expect("Get decrypted messages");

        assert_eq!(
            vault_messages.len(),
            1,
            "Expected exactly 1 U2J message, but got {}",
            vault_messages.len()
        );

        let u2j_message = match &vault_messages[0] {
            VaultMessage::U2J(ref m) => m,
            _ => panic!("Expected U2J message"),
        };

        assert_eq!(u2j_message.message, USER_MESSAGE);
    }
}
