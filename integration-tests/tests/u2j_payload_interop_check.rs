use client::commands::user::messages::send_user_to_journalist_cover_message;
#[allow(deprecated)]
use common::protocol::user::{
    encrypt_real_authenticated_message_from_user_to_journalist_via_covernode,
    encrypt_real_message_from_user_to_journalist_via_covernode,
};
use common::{FixedSizeAuthenticatedMessageText, FixedSizeMessageText};
use coverdrop_service::JournalistCoverDropService;
use integration_tests::api_wrappers::get_journalist_dead_drops;
use integration_tests::{
    api_wrappers::get_and_verify_public_keys,
    dev_u2j_mixing_config,
    stack::{CoverDropStack, StackProfile},
};
use journalist_vault::VaultMessage;
use std::time::Duration;

static OLD_FORMAT_USER_MESSAGE: &str = "This is an old format test message";
static NEW_FORMAT_USER_MESSAGE: &str = "This is a new format test message";

/// This test verifies that duplicate U2J messages are deduplicated in the CoverNode.
#[tokio::test]
#[allow(clippy::await_holding_refcell_ref)]
async fn u2j_payload_interop_check() {
    integration_tests::utils::init_logger();

    let stack = CoverDropStack::new(StackProfile::CoverDropOnly).await;

    let anchor_org_pks = stack.keys().anchor_org_pks();

    let keys_and_profiles =
        get_and_verify_public_keys(stack.api_client_cached(), &anchor_org_pks, stack.now()).await;

    //
    // User sends two messages, one plain and one authenticated
    //
    {
        let journalist_vault = stack.load_static_journalist_vault().await;
        let user_mailbox = stack.mailboxes().user();

        let journalist_id = journalist_vault
            .journalist_id()
            .await
            .expect("Get the journalist ID");

        let user_key_pair = user_mailbox.user_key_pair();

        let message = FixedSizeMessageText::new(OLD_FORMAT_USER_MESSAGE).unwrap();

        #[allow(deprecated)]
        let encrypted_outer_msg = encrypt_real_message_from_user_to_journalist_via_covernode(
            &keys_and_profiles.keys,
            user_key_pair.public_key(),
            &journalist_id,
            message.clone(),
        )
        .expect("Encrypt real message from user to journalist");

        stack
            .messaging_client()
            .post_user_message(encrypted_outer_msg)
            .await
            .expect("Send U2J message");

        let message = FixedSizeAuthenticatedMessageText::new(NEW_FORMAT_USER_MESSAGE).unwrap();

        let encrypted_outer_msg =
            encrypt_real_authenticated_message_from_user_to_journalist_via_covernode(
                &keys_and_profiles.keys,
                user_key_pair,
                &journalist_id,
                message,
            )
            .expect("Encrypt real message from user to journalist");

        stack
            .messaging_client()
            .post_user_message(encrypted_outer_msg)
            .await
            .expect("Send U2JA message");

        // trigger u2j dead drop
        for _ in 0..(dev_u2j_mixing_config().threshold_max - 2) {
            send_user_to_journalist_cover_message(
                stack.messaging_client(),
                &keys_and_profiles.keys,
            )
            .await
            .expect("Send user cover message");
        }

        tokio::time::sleep(Duration::from_secs(15)).await;
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

        let dead_drop_count = get_journalist_dead_drops(stack.api_client_cached(), 0)
            .await
            .dead_drops
            .len();

        assert_eq!(dead_drop_count, 1);

        let pulled_and_decrypted_messages = journalist_service
            .pull_and_decrypt_dead_drops(&public_keys, None::<fn(usize)>, stack.now())
            .await
            .expect("Pull and decrypt dead drops");

        // Assert that two messages have been added to the dead drop
        assert_eq!(
            pulled_and_decrypted_messages.len(),
            2,
            "Expected exactly 1 U2J message, but got {}",
            pulled_and_decrypted_messages.len()
        );

        let old_messages: Vec<_> = pulled_and_decrypted_messages
            .iter()
            .filter(|message| {
                message
                    .payload
                    .to_string()
                    .expect("Message to convert to string")
                    == OLD_FORMAT_USER_MESSAGE
            })
            .collect();

        assert_eq!(old_messages.iter().len(), 1);

        let new_messages: Vec<_> = pulled_and_decrypted_messages
            .iter()
            .filter(|message| {
                message
                    .payload
                    .to_string()
                    .expect("Message to convert to string")
                    == NEW_FORMAT_USER_MESSAGE
            })
            .collect();

        assert_eq!(new_messages.iter().len(), 1);

        let vault_messages = journalist_vault
            .messages()
            .await
            .expect("Get decrypted messages");

        assert_eq!(
            vault_messages.len(),
            2,
            "Expected exactly 2 U2J message, but got {}",
            vault_messages.len()
        );

        let old_messages: Vec<_> = vault_messages
            .iter()
            .filter(|message| match message {
                VaultMessage::U2J(ref m) => m.message == OLD_FORMAT_USER_MESSAGE,
                _ => false,
            })
            .collect();

        assert_eq!(old_messages.iter().len(), 1);

        let new_messages: Vec<_> = vault_messages
            .iter()
            .filter(|message| match message {
                VaultMessage::U2J(ref m) => m.message == NEW_FORMAT_USER_MESSAGE,
                _ => false,
            })
            .collect();

        assert_eq!(new_messages.iter().len(), 1);
    }
}
