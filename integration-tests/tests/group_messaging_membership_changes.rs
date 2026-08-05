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
async fn test_adding_and_removing_members() {
    let stack = CoverDropStack::builder(StackProfile::GroupMessagingOnly)
        .build()
        .await;

    let (alice_client, _alice_vault, _alice_coverdrop_service) =
        create_client(&stack, "alice").await;
    let (bob_client, _bob_vault, _bob_coverdrop_service) = create_client(&stack, "bob").await;
    let (charlie_client, _charlie_vault, _charlie_coverdrop_service) =
        create_client(&stack, "charlie").await;

    alice_client
        .register(5)
        .await
        .expect("Alice registration failed");
    bob_client
        .register(5)
        .await
        .expect("Bob registration failed");
    charlie_client
        .register(5)
        .await
        .expect("Charlie registration failed");

    // Alice publishes additional key packages (needed for re-add after removal)
    alice_client
        .publish_key_packages(3)
        .await
        .expect("Alice failed to publish additional key packages");

    let public_keys = get_and_verify_public_keys(
        stack.api_client_uncached(),
        &stack.keys().anchor_org_pks(),
        stack.now(),
    )
    .await
    .keys;

    // ── Create group with Alice and Bob ──────────────────

    let group_id = GroupId::new("members_test_group".to_string());
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

    let alice_msgs = receive_and_return_unread(&alice_client, &group_id, &public_keys, 0, 1).await;
    assert_eq!(alice_msgs.len(), 0);

    let bob_msgs = receive_and_return_unread(&bob_client, &group_id, &public_keys, 1, 1).await;
    assert_eq!(bob_msgs.len(), 1);

    // ── Add Charlie to the group ─────────────────────────

    alice_client
        .modify_group(
            &group_id,
            vec![
                alice_client.client_id().await.clone(),
                bob_client.client_id().await.clone(),
                charlie_client.client_id().await.clone(),
            ],
            &"Test Group".to_string(),
            &"A group for testing".to_string(),
            &public_keys,
        )
        .await
        .expect("Alice failed to add Charlie to group");

    // Alice's UsersAdded(charlie) is stored as read
    let alice_msgs = receive_and_return_unread(&alice_client, &group_id, &public_keys, 0, 2).await;
    assert_eq!(alice_msgs.len(), 0);
    assert_group_info(
        &alice_client,
        &group_id,
        "Test Group",
        "A group for testing",
        &[
            &alice_client.client_id().await,
            &bob_client.client_id().await,
            &charlie_client.client_id().await,
        ],
    )
    .await;

    // Charlie receives the Welcome (not stored) + GroupInfo message
    let charlie_msgs =
        receive_and_return_unread(&charlie_client, &group_id, &public_keys, 1, 1).await;
    assert_eq!(charlie_msgs.len(), 1);
    assert_eq!(charlie_msgs[0].sender, alice_client.client_id().await);
    assert_eq!(
        charlie_msgs[0].content,
        GroupMessageContent::GroupInfo {
            display_name: "Test Group".to_string(),
            description: "A group for testing".to_string(),
        }
    );

    // Bob receives the add-member commit (not stored) + UsersAdded message
    let bob_msgs = receive_and_return_unread(&bob_client, &group_id, &public_keys, 1, 2).await;
    assert_eq!(bob_msgs.len(), 1);
    assert_eq!(
        bob_msgs[0].content,
        GroupMessageContent::UsersAdded(vec![charlie_client.client_id().await.clone()])
    );

    // ── Remove Alice and change group info ───────────────
    // Bob removes Alice and changes the group name/description.
    // Alice should NOT receive the group info change messages (remove happens first).

    bob_client
        .modify_group(
            &group_id,
            vec![
                bob_client.client_id().await.clone(),
                charlie_client.client_id().await.clone(),
            ],
            &"Final Group Name".to_string(),
            &"Final description".to_string(),
            &public_keys,
        )
        .await
        .expect("Bob failed to remove Alice from the group");

    // Bob's UsersRemoved and group info change messages are stored as read
    let bob_msgs = receive_and_return_unread(&bob_client, &group_id, &public_keys, 0, 5).await;
    assert_eq!(bob_msgs.len(), 0);

    // Alice receives the commit removing her, and a UsersRemoved message,
    // but NOT the group info change messages (she was removed first).
    let alice_msgs = receive_and_return_unread(&alice_client, &group_id, &public_keys, 1, 3).await;
    assert_eq!(alice_msgs.len(), 1);
    assert_eq!(alice_msgs[0].sender, bob_client.client_id().await);
    assert_eq!(
        alice_msgs[0].content,
        GroupMessageContent::UsersRemoved(vec![alice_client.client_id().await.clone()])
    );
    assert!(
        !alice_client
            .has_active_mls_group(&group_id)
            .await
            .expect("Failed to check if Alice has active MLS group"),
        "Alice's MLS group should be inactive after being removed"
    );

    // Bob sends a message to the group after removing Alice
    let bob_post_removal_message = MlsMessageContentWithId::new_from_content(
        MlsMessageContent::Text("This message is only for Charlie!".to_string()),
    );
    bob_client
        .send_message(&group_id, &bob_post_removal_message)
        .await
        .expect("Bob failed to send message after removing Alice");

    // Charlie receives the removal commit (UsersRemoved), group info changes, and Bob's message
    let charlie_msgs =
        receive_and_return_unread(&charlie_client, &group_id, &public_keys, 4, 5).await;
    assert_eq!(charlie_msgs.len(), 4);
    assert_eq!(charlie_msgs[0].sender, bob_client.client_id().await);
    assert_eq!(
        charlie_msgs[0].content,
        GroupMessageContent::UsersRemoved(vec![alice_client.client_id().await.clone()])
    );
    assert_eq!(charlie_msgs[1].sender, bob_client.client_id().await);
    assert_eq!(
        charlie_msgs[1].content,
        GroupMessageContent::GroupNameChanged("Final Group Name".to_string())
    );
    assert_eq!(charlie_msgs[2].sender, bob_client.client_id().await);
    assert_eq!(
        charlie_msgs[2].content,
        GroupMessageContent::GroupDescriptionChanged("Final description".to_string())
    );
    assert_eq!(charlie_msgs[3].sender, bob_client.client_id().await);
    assert_eq!(
        charlie_msgs[3].content,
        GroupMessageContent::Text("This message is only for Charlie!".to_string())
    );

    // Alice should not receive any new messages
    let alice_msgs = receive_and_return_unread(&alice_client, &group_id, &public_keys, 0, 3).await;
    assert_eq!(alice_msgs.len(), 0);

    // ── Re-add Alice ─────────────────────────────────────
    // Bob (who joined via Welcome) adds Alice back. This tests that Bob's
    // Welcome message includes the ratchet tree extension.

    bob_client
        .modify_group(
            &group_id,
            vec![
                alice_client.client_id().await.clone(),
                bob_client.client_id().await.clone(),
                charlie_client.client_id().await.clone(),
            ],
            &"Final Group Name".to_string(),
            &"Final description".to_string(),
            &public_keys,
        )
        .await
        .expect("Bob failed to add Alice back to the group");

    // Bob's UsersAdded(alice) is stored as read
    let bob_msgs = receive_and_return_unread(&bob_client, &group_id, &public_keys, 0, 7).await;
    assert_eq!(bob_msgs.len(), 0);

    // Alice receives the Welcome (not stored) + GroupInfo message.
    // Alice's old messages from before her removal are still in the vault.
    let alice_msgs = receive_and_return_unread(&alice_client, &group_id, &public_keys, 1, 4).await;
    assert_eq!(alice_msgs.len(), 1);
    assert_eq!(alice_msgs[0].sender, bob_client.client_id().await);
    assert_eq!(
        alice_msgs[0].content,
        GroupMessageContent::GroupInfo {
            display_name: "Final Group Name".to_string(),
            description: "Final description".to_string(),
        }
    );
    assert_group_info(
        &alice_client,
        &group_id,
        "Final Group Name",
        "Final description",
        &[
            &alice_client.client_id().await,
            &bob_client.client_id().await,
            &charlie_client.client_id().await,
        ],
    )
    .await;

    // Charlie receives UsersAdded(alice)
    let charlie_msgs =
        receive_and_return_unread(&charlie_client, &group_id, &public_keys, 1, 6).await;
    assert_eq!(charlie_msgs.len(), 1);
    assert_eq!(charlie_msgs[0].sender, bob_client.client_id().await);
    assert_eq!(
        charlie_msgs[0].content,
        GroupMessageContent::UsersAdded(vec![alice_client.client_id().await.clone()])
    );

    // Alice should NOT have access to messages sent while she was removed.
    let alice_groups = alice_client
        .get_groups_and_messages()
        .await
        .expect("Failed to get Alice's groups");
    let alice_group = alice_groups
        .iter()
        .find(|g| g.id() == &group_id)
        .expect("Group not found for Alice");
    assert!(
        !alice_group.messages().iter().any(|m| m.content
            == GroupMessageContent::Text("This message is only for Charlie!".to_string())),
        "Alice should not have messages sent while she was removed"
    );

    // Alice can now send messages to the group again
    let alice_return_message = MlsMessageContentWithId::new_from_content(MlsMessageContent::Text(
        "I'm back! Did I miss anything?".to_string(),
    ));
    alice_client
        .send_message(&group_id, &alice_return_message)
        .await
        .expect("Alice failed to send message after being re-added");

    let public_keys = get_and_verify_public_keys(
        stack.api_client_uncached(),
        &stack.keys().anchor_org_pks(),
        stack.now(),
    )
    .await
    .keys;

    // Charlie receives Alice's message
    let charlie_msgs =
        receive_and_return_unread(&charlie_client, &group_id, &public_keys, 1, 7).await;
    assert_eq!(charlie_msgs.len(), 1);
    assert_eq!(charlie_msgs[0].sender, alice_client.client_id().await);
    assert_eq!(
        charlie_msgs[0].content,
        GroupMessageContent::Text("I'm back! Did I miss anything?".to_string())
    );

    // Bob receives Alice's message
    let bob_msgs = receive_and_return_unread(&bob_client, &group_id, &public_keys, 1, 8).await;
    assert_eq!(bob_msgs.len(), 1);
    assert_eq!(bob_msgs[0].sender, alice_client.client_id().await);
    assert_eq!(
        bob_msgs[0].content,
        GroupMessageContent::Text("I'm back! Did I miss anything?".to_string())
    );

    // Bob sends a message to the full group — Alice can receive it
    let bob_welcome_back = MlsMessageContentWithId::new_from_content(MlsMessageContent::Text(
        "Welcome back Alice!".to_string(),
    ));
    bob_client
        .send_message(&group_id, &bob_welcome_back)
        .await
        .expect("Bob failed to send welcome back message");

    // Alice receives Bob's message (her own sent message is already read)
    let alice_msgs = receive_and_return_unread(&alice_client, &group_id, &public_keys, 1, 6).await;
    assert_eq!(alice_msgs.len(), 1);
    assert_eq!(alice_msgs[0].sender, bob_client.client_id().await);
    assert_eq!(
        alice_msgs[0].content,
        GroupMessageContent::Text("Welcome back Alice!".to_string())
    );
}
