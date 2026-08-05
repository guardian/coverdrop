use group_messaging_service::MlsMessageContent;
use group_messaging_service::MlsMessageContentWithId;
use integration_tests::api_wrappers::get_and_verify_public_keys;
use integration_tests::group_messaging_utils::{create_client, receive_and_return_unread};
use integration_tests::stack::{CoverDropStack, StackProfile};
use journalist_vault::GroupId;
use journalist_vault::GroupMessageContent;

#[tokio::test(flavor = "multi_thread")]
async fn test_key_rotation() {
    let stack = CoverDropStack::builder(StackProfile::GroupMessagingOnly)
        .build()
        .await;

    let (alice_client, alice_vault, alice_coverdrop_service) = create_client(&stack, "alice").await;
    let (bob_client, _bob_vault, _bob_coverdrop_service) = create_client(&stack, "bob").await;

    alice_client
        .register(5)
        .await
        .expect("Alice registration failed");
    bob_client
        .register(5)
        .await
        .expect("Bob registration failed");

    let public_keys = get_and_verify_public_keys(
        stack.api_client_uncached(),
        &stack.keys().anchor_org_pks(),
        stack.now(),
    )
    .await
    .keys;

    // Create group with Alice and Bob
    let group_id = GroupId::new("key_rotation_group".to_string());
    alice_client
        .create_group_with_members(
            group_id.clone(),
            vec![
                alice_client.client_id().await.clone(),
                bob_client.client_id().await.clone(),
            ],
            "Key Rotation Test",
            "Testing key rotation",
            &public_keys,
        )
        .await
        .expect("Alice failed to create group");

    let alice_msgs = receive_and_return_unread(&alice_client, &group_id, &public_keys, 0, 1).await;
    assert_eq!(alice_msgs.len(), 0);

    let bob_msgs = receive_and_return_unread(&bob_client, &group_id, &public_keys, 1, 1).await;
    assert_eq!(bob_msgs.len(), 1);

    // ── Key rotation ─────────────────────────────────────

    // Alice rotates her identity key pair. Bob can still communicate securely.
    alice_coverdrop_service
        .rotate_sentinel_id_key(stack.now())
        .await
        .expect("Alice failed to rotate identity key");
    let alice_new_id_key_pair = alice_vault
        .latest_sentinel_id_key_pair(stack.now())
        .await
        .expect("Get latest identity key pair after rotation")
        .expect("Alice should have an identity key pair in the vault after rotation");
    // assert that the new key pair is published
    assert!(
        alice_vault
            .sentinel_id_key_pairs(stack.now())
            .await
            .expect("got alice's id key pairs")
            .any(|kp| kp.public_key() == alice_new_id_key_pair.public_key()),
        "Alice's new identity key pair should be in the vault after rotation"
    );

    alice_client
        .rotate_signature_key(alice_new_id_key_pair)
        .await
        .expect("Alice failed to update her leaf node");

    alice_client
        .publish_key_packages(5)
        .await
        .expect("Alice failed to publish additional key packages after rotation");

    // Alice sends a message after key rotation
    let alice_message_after_rotation =
        MlsMessageContentWithId::new_from_content(MlsMessageContent::Text(
            "Hello again, this is Alice after rotating her identity key!".to_string(),
        ));
    alice_client
        .send_message(&group_id, &alice_message_after_rotation)
        .await
        .expect("Alice failed to send message after key rotation");

    // Alice's own post-rotation message is stored as read
    let alice_msgs = receive_and_return_unread(&alice_client, &group_id, &public_keys, 0, 2).await;
    assert_eq!(alice_msgs.len(), 0);

    // fetch public key hierarchy again
    let public_keys = get_and_verify_public_keys(
        stack.api_client_uncached(),
        &stack.keys().anchor_org_pks(),
        stack.now(),
    )
    .await
    .keys;

    // Bob receives Alice's post-rotation message
    let bob_msgs = receive_and_return_unread(&bob_client, &group_id, &public_keys, 1, 2).await;
    assert_eq!(bob_msgs.len(), 1);
    assert_eq!(
        bob_msgs[0].content,
        GroupMessageContent::Text(
            "Hello again, this is Alice after rotating her identity key!".to_string()
        )
    );
}
