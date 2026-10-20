use chrono::{DateTime, Utc};
use client::commands::user::messages::send_user_to_journalist_cover_message;
use common::protocol::user::encrypt_real_authenticated_message_from_user_to_journalist_via_covernode;
use common::FixedSizeAuthenticatedMessageText;
use coverdrop_service::JournalistCoverDropService;
use integration_tests::api_wrappers::get_user_to_journalist_dead_drops;
use integration_tests::{
    api_wrappers::get_and_verify_public_keys,
    dev_u2j_mixing_config,
    stack::{CoverDropStack, StackProfile},
};
use std::time::Duration;

/// This test verifies that malformed U2J messages are filtered out in Sentinel.
#[tokio::test]
#[allow(clippy::await_holding_refcell_ref)]
async fn sentinel_handle_malformed_u2j_scenario() {
    integration_tests::utils::init_logger();

    let stack = CoverDropStack::new(StackProfile::CoverDropOnly).await;

    let anchor_org_pks = stack.keys().anchor_org_pks();

    let keys_and_profiles =
        get_and_verify_public_keys(stack.api_client_cached(), &anchor_org_pks, stack.now()).await;

    //
    // User sends the malformed real message, plus cover messages to trigger a dead drop
    //
    {
        let journalist_vault = stack.load_static_journalist_vault().await;
        let user_mailbox = stack.mailboxes().user();

        let journalist_id = journalist_vault
            .journalist_id()
            .await
            .expect("Get the journalist ID");

        let user_key_pair = user_mailbox.user_key_pair();

        let malformed_message =
            FixedSizeAuthenticatedMessageText::from_vec_unchecked(vec![1_u8; 472]);

        let encrypted_outer_msg =
            encrypt_real_authenticated_message_from_user_to_journalist_via_covernode(
                &keys_and_profiles.keys,
                user_key_pair,
                &journalist_id,
                malformed_message,
            )
            .expect("Encrypt real message from user to journalist");

        // Send malformed message
        stack
            .messaging_client()
            .post_user_message(encrypted_outer_msg.clone())
            .await
            .expect("Send U2J message");

        // trigger u2j dead drop
        for _ in 0..(dev_u2j_mixing_config().threshold_max - 1) {
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
    // Journalist fetches dead drops and should ignore malformed u2j messages
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

        let dead_drop_count = get_user_to_journalist_dead_drops(
            stack.api_client_cached(),
            DateTime::<Utc>::UNIX_EPOCH,
        )
        .await
        .dead_drops
        .len();

        assert_eq!(dead_drop_count, 1);

        let pulled_and_decrypted_messages = journalist_service
            .pull_and_decrypt_dead_drops(&public_keys, None::<fn(usize)>, stack.now())
            .await
            .expect("Pull and decrypt dead drops");

        // Assert that no messages are decrypted and valid from the dead drop
        assert_eq!(
            pulled_and_decrypted_messages.len(),
            0,
            "Expected no U2J messages, but got {}",
            pulled_and_decrypted_messages.len()
        );

        // Assert that no messages are added to the journalist's vault
        let vault_messages = journalist_vault
            .messages()
            .await
            .expect("Get decrypted messages");

        assert_eq!(
            vault_messages.len(),
            0,
            "Expected no U2J messages, but got {}",
            vault_messages.len()
        );
    }
}
