use group_messaging_service::MlsMessageContent;
use group_messaging_service::MlsMessageContentWithId;
use integration_tests::api_wrappers::get_and_verify_public_keys;
use integration_tests::group_messaging_utils::{
    assert_group_info, create_client, receive_and_return_unread,
};
use integration_tests::stack::{CoverDropStack, StackProfile};
use journalist_vault::GroupId;
use journalist_vault::GroupMessageContent;

#[tokio::test(flavor = "multi_thread")]
async fn test_basic_messaging() {
    let stack = CoverDropStack::builder(StackProfile::GroupMessagingOnly)
        .build()
        .await;

    let (alice_client, _alice_vault, _alice_coverdrop_service) =
        create_client(&stack, "alice").await;
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

    // ── Group creation and basic messaging ───────────────

    let group_id = GroupId::new("basic_messaging_group".to_string());
    alice_client
        .create_group_with_members(
            group_id.clone(),
            vec![
                alice_client.client_id().await.clone(),
                bob_client.client_id().await.clone(),
            ],
            "Test Group",
            "A group for testing",
            &public_keys,
        )
        .await
        .expect("Alice failed to create group with Bob");

    // Alice's GroupInfo message is stored as read (sender-stored messages are pre-read)
    let alice_msgs = receive_and_return_unread(&alice_client, &group_id, &public_keys, 0, 1).await;
    assert_eq!(alice_msgs.len(), 0);

    // Bob receives the Welcome (not stored) + GroupInfo message
    let bob_msgs = receive_and_return_unread(&bob_client, &group_id, &public_keys, 1, 1).await;
    assert_eq!(bob_msgs.len(), 1);
    assert_eq!(bob_msgs[0].sender, alice_client.client_id().await);
    assert_eq!(
        bob_msgs[0].content,
        GroupMessageContent::GroupInfo {
            display_name: "Test Group".to_string(),
            description: "A group for testing".to_string(),
        }
    );
    assert_group_info(
        &bob_client,
        &group_id,
        "Test Group",
        "A group for testing",
        &[
            &alice_client.client_id().await,
            &bob_client.client_id().await,
        ],
    )
    .await;

    // Alice sends a message to the group
    let alice_message = MlsMessageContentWithId::new_from_content(MlsMessageContent::Text(
        "Hello Bob, this is a test message from Alice!".to_string(),
    ));
    alice_client
        .send_message(&group_id, &alice_message)
        .await
        .expect("Alice failed to send message");

    // Bob receives Alice's message
    let bob_msgs = receive_and_return_unread(&bob_client, &group_id, &public_keys, 1, 2).await;
    assert_eq!(bob_msgs.len(), 1);
    assert_eq!(bob_msgs[0].sender, alice_client.client_id().await);
    assert_eq!(
        bob_msgs[0].content,
        GroupMessageContent::Text("Hello Bob, this is a test message from Alice!".to_string())
    );

    // Bob sends a reply to Alice
    let bob_message = MlsMessageContentWithId::new_from_content(MlsMessageContent::Text(
        "Hi Alice! Message received successfully!".to_string(),
    ));
    bob_client
        .send_message(&group_id, &bob_message)
        .await
        .expect("Bob failed to send message");

    // Alice receives Bob's reply (her own sent message is already read)
    let alice_msgs = receive_and_return_unread(&alice_client, &group_id, &public_keys, 1, 3).await;
    assert_eq!(alice_msgs.len(), 1);
    assert_eq!(alice_msgs[0].sender, bob_client.client_id().await);
    assert_eq!(
        alice_msgs[0].content,
        GroupMessageContent::Text("Hi Alice! Message received successfully!".to_string())
    );

    // Alice and Bob's copy of the same message have the same published_at timestamp.
    let alice_groups = alice_client
        .get_groups_and_messages()
        .await
        .expect("Failed to get Alice's groups");
    let alice_group = alice_groups.iter().find(|g| g.id() == &group_id).unwrap();
    let alice_stored_msg = alice_group
        .messages()
        .iter()
        .find(|m| m.id == alice_message.id)
        .expect("Alice should have her own sent message stored");
    assert_eq!(
        alice_stored_msg.published_at.timestamp_millis(),
        bob_msgs[0].published_at.timestamp_millis(),
        "Alice and Bob's copy of the message should have the same published_at timestamp"
    );

    // ── Modify group info ────────────────────────────────

    alice_client
        .modify_group(
            &group_id,
            vec![
                alice_client.client_id().await.clone(),
                bob_client.client_id().await.clone(),
            ],
            &"Renamed Group".to_string(),
            &"A renamed group for testing".to_string(),
            &public_keys,
        )
        .await
        .expect("Alice failed to modify group info");

    // Alice's GroupNameChanged + GroupDescriptionChanged stored as read
    let alice_msgs = receive_and_return_unread(&alice_client, &group_id, &public_keys, 0, 5).await;
    assert_eq!(alice_msgs.len(), 0);
    assert_group_info(
        &alice_client,
        &group_id,
        "Renamed Group",
        "A renamed group for testing",
        &[
            &alice_client.client_id().await,
            &bob_client.client_id().await,
        ],
    )
    .await;

    // Bob receives GroupNameChanged + GroupDescriptionChanged
    let bob_msgs = receive_and_return_unread(&bob_client, &group_id, &public_keys, 2, 5).await;
    assert_eq!(bob_msgs.len(), 2);
    assert_eq!(bob_msgs[0].sender, alice_client.client_id().await);
    assert_eq!(
        bob_msgs[0].content,
        GroupMessageContent::GroupNameChanged("Renamed Group".to_string())
    );
    assert_eq!(bob_msgs[1].sender, alice_client.client_id().await);
    assert_eq!(
        bob_msgs[1].content,
        GroupMessageContent::GroupDescriptionChanged("A renamed group for testing".to_string())
    );
    assert_group_info(
        &bob_client,
        &group_id,
        "Renamed Group",
        "A renamed group for testing",
        &[
            &alice_client.client_id().await,
            &bob_client.client_id().await,
        ],
    )
    .await;

    // ── Typing indicator messages ────────────────────────

    let typing_start = MlsMessageContentWithId::new_from_content(MlsMessageContent::IsTyping(true));
    alice_client
        .send_message(&group_id, &typing_start)
        .await
        .expect("Alice failed to send IsTyping(true)");

    // Bob receives the typing indicator
    let bob_msgs = receive_and_return_unread(&bob_client, &group_id, &public_keys, 1, 6).await;
    assert_eq!(bob_msgs.len(), 1);
    assert_eq!(bob_msgs[0].sender, alice_client.client_id().await);
    assert_eq!(bob_msgs[0].content, GroupMessageContent::IsTyping(true));

    let typing_stop = MlsMessageContentWithId::new_from_content(MlsMessageContent::IsTyping(false));
    alice_client
        .send_message(&group_id, &typing_stop)
        .await
        .expect("Alice failed to send IsTyping(false)");

    // Bob receives the typing stopped indicator
    let bob_msgs = receive_and_return_unread(&bob_client, &group_id, &public_keys, 1, 7).await;
    assert_eq!(bob_msgs.len(), 1);
    assert_eq!(bob_msgs[0].sender, alice_client.client_id().await);
    assert_eq!(bob_msgs[0].content, GroupMessageContent::IsTyping(false));

    // ── Message read status ──────────────────────────────

    // All of Bob's messages should be read after previous receive_and_return_unread calls.
    let bob_groups = bob_client
        .get_groups_and_messages()
        .await
        .expect("Failed to get Bob's groups");
    let bob_group = bob_groups
        .iter()
        .find(|g| g.id() == &group_id)
        .expect("Group not found");
    let bob_messages = bob_group.messages();
    assert!(
        bob_messages.iter().all(|m| m.read),
        "All messages should be read after receive_and_return_unread calls"
    );

    // Mark some messages as unread
    let message_ids_to_mark: Vec<_> = bob_messages.iter().take(3).map(|m| m.id).collect();
    bob_client
        .update_group_messages_read_status(&group_id, message_ids_to_mark.clone(), false)
        .await
        .expect("Failed to mark messages as unread");

    // Verify only those messages are now unread
    let bob_groups = bob_client
        .get_groups_and_messages()
        .await
        .expect("Failed to get Bob's groups after marking as unread");
    let bob_group = bob_groups
        .iter()
        .find(|g| g.id() == &group_id)
        .expect("Group not found");
    let bob_messages = bob_group.messages();
    for msg in bob_messages {
        if message_ids_to_mark.contains(&msg.id) {
            assert!(!msg.read, "Message {} should be marked as unread", msg.id);
        } else {
            assert!(msg.read, "Message {} should still be read", msg.id);
        }
    }

    // Set messages back to read
    bob_client
        .update_group_messages_read_status(&group_id, message_ids_to_mark.clone(), true)
        .await
        .expect("Failed to mark messages as read");

    let bob_groups = bob_client
        .get_groups_and_messages()
        .await
        .expect("Failed to get Bob's groups after marking as read");
    let bob_group = bob_groups
        .iter()
        .find(|g| g.id() == &group_id)
        .expect("Group not found");
    let bob_messages = bob_group.messages();
    assert!(
        bob_messages.iter().all(|m| m.read),
        "All messages should be read again after marking as read"
    );

    // ── Message size limit ───────────────────────────────

    // A message exceeding MAX_MESSAGE_SIZE_BYTES (16 KB) should be rejected on the client side
    let oversized_text = "x".repeat(16 * 1024);
    let oversized_message =
        MlsMessageContentWithId::new_from_content(MlsMessageContent::Text(oversized_text));
    let result = alice_client
        .send_message(&group_id, &oversized_message)
        .await;
    assert!(
        result.is_err(),
        "Sending a message exceeding the size limit should fail"
    );
    assert!(
        result.unwrap_err().to_string().contains("exceeds maximum"),
        "Error message should mention the size limit"
    );
}
