use common::api::models::journalist_id::JournalistIdentity;
use coverdrop_service::JournalistCoverDropService;
use integration_tests::api_wrappers::{generate_test_journalist, get_and_verify_public_keys};
use integration_tests::group_messaging_utils::TestClient;
use integration_tests::secrets::MAILBOX_PASSWORD;
use integration_tests::stack::{CoverDropStack, StackProfile};
use journalist_vault::JournalistVault;

async fn create_client(
    stack: &CoverDropStack,
    journalist_id_str: &str,
) -> (TestClient, JournalistVault, JournalistCoverDropService) {
    generate_test_journalist(
        stack.api_client_cached(),
        stack.keys_path(),
        stack.temp_dir_path(),
        stack.now(),
        stack.trust_anchors(),
        Some(journalist_id_str.to_string()),
    )
    .await;

    let journalist_identity =
        JournalistIdentity::new(journalist_id_str).expect("Failed to create journalist identity");

    let vault_path = stack
        .temp_dir_path()
        .join(format!("{}.vault", journalist_id_str));

    let vault = JournalistVault::open(&vault_path, MAILBOX_PASSWORD, stack.trust_anchors())
        .await
        .expect("Load journalist vault");
    let identity_key_pair = vault
        .latest_id_key_pair(stack.now())
        .await
        .expect("Get latest identity key pair")
        .expect("Journalist should have an identity key pair in the vault");

    let test_client = TestClient::new(
        &journalist_identity,
        identity_key_pair,
        stack.delivery_service_url().await.clone(),
    );

    let journalist_coverdrop_service =
        JournalistCoverDropService::new(stack.api_client_cached(), &vault);

    (test_client, vault, journalist_coverdrop_service)
}

#[tokio::test]
async fn test_clients_group_messaging() {
    // Start the stack with delivery service enabled
    let stack = CoverDropStack::builder(StackProfile::GroupMessagingOnly)
        .build()
        .await;

    // create three clients with associated vaults and identities
    let (mut alice_client, alice_vault, alice_coverdrop_service) =
        create_client(&stack, "alice").await;
    let (mut bob_client, _bob_vault, _bob_coverdrop_service) = create_client(&stack, "bob").await;
    let (mut charlie_client, _charlie_vault, _charlie_coverdrop_service) =
        create_client(&stack, "charlie").await;

    // Register key packages for all clients
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

    // Alice can publish a few more key packages after registration as well
    alice_client
        .publish_key_packages(3)
        .await
        .expect("Alice failed to publish additional key packages");

    let public_keys = get_and_verify_public_keys(
        stack.api_client_uncached(),
        &stack.trust_anchors(),
        stack.now(),
    )
    .await
    .keys;

    // Alice requests the list of clients
    let clients = alice_client
        .get_clients()
        .await
        .expect("Alice failed to get client list");
    assert_eq!(clients.len(), 3, "Should have 3 registered clients");
    assert!(
        clients.contains(&alice_client.client_id),
        "Alice should be in the client list"
    );
    assert!(
        clients.contains(&bob_client.client_id),
        "Bob should be in the client list"
    );
    assert!(
        clients.contains(&charlie_client.client_id),
        "Charlie should be in the client list"
    );

    // Create a group with Alice as the creator
    let group_id = b"test_group_001".to_vec();
    alice_client
        .create_group(group_id.clone())
        .expect("Alice failed to create group");

    // Alice gets Bob's key package
    let bob_key_package = alice_client
        .get_key_package(&bob_client.client_id, &public_keys)
        .await
        .expect("Failed to get Bob's key package");

    // Alice adds Bob to the group
    alice_client
        .add_members(&group_id, vec![bob_key_package])
        .await
        .expect("Alice failed to add Bob to group");

    // Bob receives the Welcome message and joins the group
    let bob_messages = bob_client
        .receive_messages(&public_keys)
        .await
        .expect("Bob failed to receive Welcome message");

    // Welcome messages don't produce application messages, just group joins
    assert_eq!(
        bob_messages.len(),
        0,
        "Welcome message should not produce application messages"
    );

    // Verify Bob is now in the group
    assert!(
        bob_client.groups.contains_key(&group_id),
        "Bob should have joined the group"
    );

    // Alice sends a message to the group
    let alice_message = b"Hello Bob, this is a test message from Alice!";
    alice_client
        .send_message(&group_id, alice_message)
        .await
        .expect("Alice failed to send message");

    // Bob receives Alice's message
    let bob_messages = bob_client
        .receive_messages(&public_keys)
        .await
        .expect("Bob failed to receive Alice's message");

    // Verify Bob received the correct message
    assert_eq!(
        bob_messages.len(),
        1,
        "Bob should have received exactly one application message"
    );
    assert_eq!(
        bob_messages[0], alice_message,
        "Bob's received message should match Alice's sent message"
    );

    // Bob sends a reply to Alice
    let bob_message = b"Hi Alice! Message received successfully!";
    bob_client
        .send_message(&group_id, bob_message)
        .await
        .expect("Bob failed to send message");

    // Alice receives Bob's reply
    let alice_messages = alice_client
        .receive_messages(&public_keys)
        .await
        .expect("Alice failed to receive Bob's message");

    // Verify Alice received Bob's reply
    assert_eq!(
        alice_messages.len(),
        1,
        "Alice should have received exactly one application message"
    );
    assert_eq!(
        alice_messages[0], bob_message,
        "Alice's received message should match Bob's sent message"
    );

    // Alice adds Charlie to the group
    let charlie_key_package = alice_client
        .get_key_package(&charlie_client.client_id, &public_keys)
        .await
        .expect("Failed to get Charlie's key package");

    alice_client
        .add_members(&group_id, vec![charlie_key_package])
        .await
        .expect("Alice failed to add Charlie to group");

    // Charlie receives the Welcome message and joins the group
    let charlie_messages = charlie_client
        .receive_messages(&public_keys)
        .await
        .expect("Charlie failed to receive Welcome message");

    // Welcome messages don't produce application messages, just group joins
    assert_eq!(
        charlie_messages.len(),
        0,
        "Welcome message should not produce application messages"
    );

    // Verify Charlie is now in the group
    assert!(
        charlie_client.groups.contains_key(&group_id),
        "Charlie should have joined the group"
    );

    // Charlie sends a message to the group
    let charlie_message = b"Hello everyone! Charlie here!";
    charlie_client
        .send_message(&group_id, charlie_message)
        .await
        .expect("Charlie failed to send message");

    // Alice receives Charlie's message
    let alice_messages = alice_client
        .receive_messages(&public_keys)
        .await
        .expect("Alice failed to receive Charlie's message");

    assert_eq!(
        alice_messages.len(),
        1,
        "Alice should have received exactly one application message from Charlie"
    );
    assert_eq!(
        alice_messages[0], charlie_message,
        "Alice's received message should match Charlie's sent message"
    );

    // Bob receives Charlie's message
    let bob_messages = bob_client
        .receive_messages(&public_keys)
        .await
        .expect("Bob failed to receive Charlie's message");

    assert_eq!(
        bob_messages.len(),
        1,
        "Bob should have received exactly one application message from Charlie"
    );
    assert_eq!(
        bob_messages[0], charlie_message,
        "Bob's received message should match Charlie's sent message"
    );

    // Alice rotates her identity key pair. The rest of the group is informed and can still communicate securely.
    // NOTE: we'll eventually need this logic to be in the SentinelMessagingService so that with a single function call we can
    // rotate the key, then update the public API, and members of all MLS groups.
    // What if one succeeds and the other fails?? Don't promote from candidate to published until both succeed?
    alice_coverdrop_service
        .rotate_id_key(stack.now())
        .await
        .expect("Alice failed to rotate identity key");
    let alice_new_id_key_pair = alice_vault
        .latest_id_key_pair(stack.now())
        .await
        .expect("Get latest identity key pair after rotation")
        .expect("Alice should have an identity key pair in the vault after rotation");
    // assert that the new key pair is published
    assert!(
        alice_vault
            .id_key_pairs(stack.now())
            .await
            .expect("got alice's id key pairs")
            .any(|kp| kp.public_key() == alice_new_id_key_pair.public_key()),
        "Alice's new identity key pair should be in the vault after rotation"
    );

    alice_client
        .rotate_signature_key(alice_new_id_key_pair)
        .await
        .expect("Alice failed to update her leaf node");

    // Alice sends another message to the group after key rotation
    let alice_message_after_rotation =
        b"Hello again, this is Alice after rotating her identity key!";
    alice_client
        .send_message(&group_id, alice_message_after_rotation)
        .await
        .expect("Alice failed to send message after key rotation");

    // Bob receives Alice's message after key rotation
    // fetch public key hierarchy again
    let public_keys = get_and_verify_public_keys(
        stack.api_client_uncached(),
        &stack.trust_anchors(),
        stack.now(),
    )
    .await
    .keys;
    let bob_messages = bob_client
        .receive_messages(&public_keys)
        .await
        .expect("Bob failed to receive Alice's message after key rotation");
    assert_eq!(
        bob_messages.len(),
        1,
        "Bob should have received exactly one application message from Alice after key rotation"
    );
    assert_eq!(
        bob_messages[0], alice_message_after_rotation,
        "Bob's received message should match Alice's sent message after key rotation"
    );
}
