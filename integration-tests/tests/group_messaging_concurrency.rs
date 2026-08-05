use std::sync::Arc;

use group_messaging_service::MlsMessageContent;
use group_messaging_service::MlsMessageContentWithId;
use integration_tests::api_wrappers::get_and_verify_public_keys;
use integration_tests::group_messaging_utils::create_client;
use integration_tests::stack::{CoverDropStack, StackProfile};
use journalist_vault::GroupId;

/// Verifies that concurrent send and receive operations don't cause SecretReuseError.
/// Alice and Bob both send 100 messages and receive concurrently.
/// The internal mutex on MLS group state prevents data races.
#[tokio::test(flavor = "multi_thread")]
async fn test_concurrent_send_and_receive() {
    let stack = CoverDropStack::builder(StackProfile::GroupMessagingOnly)
        .build()
        .await;

    let (alice_client, _alice_vault, _alice_coverdrop_service) =
        create_client(&stack, "alice_concurrent").await;
    let (bob_client, _bob_vault, _bob_coverdrop_service) =
        create_client(&stack, "bob_concurrent").await;

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

    let group_id = GroupId::new("concurrent_test_group".to_string());
    alice_client
        .create_group_with_members(
            group_id.clone(),
            vec![alice_client.client_id().await, bob_client.client_id().await],
            "Concurrency Test",
            "Testing concurrent send and receive",
            &public_keys,
        )
        .await
        .expect("Alice failed to create group");

    // Bob receives the Welcome so his group state is initialized
    bob_client
        .receive_and_store_messages(&public_keys)
        .await
        .expect("Bob failed to receive welcome");

    // Use Arc to share clients across concurrent tasks.
    let alice = Arc::new(alice_client);
    let bob = Arc::new(bob_client);
    let public_keys = Arc::new(public_keys);

    let num_send_messages = 100;

    // Alice sends 100 messages
    let alice_send = alice.clone();
    let group_id_send = group_id.clone();
    let alice_send_handle = tokio::spawn(async move {
        let mut errors = Vec::new();
        for i in 0..num_send_messages {
            let msg = MlsMessageContentWithId::new_from_content(MlsMessageContent::Text(format!(
                "Alice message {}",
                i
            )));
            if let Err(e) = alice_send.send_message(&group_id_send, &msg).await {
                errors.push(format!("Alice send {}: {}", i, e));
            }
        }
        errors
    });

    // Bob sends 100 messages
    let bob_send = bob.clone();
    let group_id_send = group_id.clone();
    let bob_send_handle = tokio::spawn(async move {
        let mut errors = Vec::new();
        for i in 0..num_send_messages {
            let msg = MlsMessageContentWithId::new_from_content(MlsMessageContent::Text(format!(
                "Bob message {}",
                i
            )));
            if let Err(e) = bob_send.send_message(&group_id_send, &msg).await {
                errors.push(format!("Bob send {}: {}", i, e));
            }
        }
        errors
    });

    // Alice receives concurrently
    let alice_recv = alice.clone();
    let public_keys_recv = public_keys.clone();
    let alice_recv_handle = tokio::spawn(async move {
        let mut errors = Vec::new();
        for i in 0..20 {
            if let Err(e) = alice_recv
                .receive_and_store_messages(&public_keys_recv)
                .await
            {
                errors.push(format!("Alice recv {}: {}", i, e));
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
        }
        errors
    });

    // Bob receives concurrently
    let bob_recv = bob.clone();
    let public_keys_recv = public_keys.clone();
    let bob_recv_handle = tokio::spawn(async move {
        let mut errors = Vec::new();
        for i in 0..20 {
            if let Err(e) = bob_recv.receive_and_store_messages(&public_keys_recv).await {
                errors.push(format!("Bob recv {}: {}", i, e));
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
        }
        errors
    });

    let (alice_send_result, bob_send_result, alice_recv_result, bob_recv_result) = tokio::join!(
        alice_send_handle,
        bob_send_handle,
        alice_recv_handle,
        bob_recv_handle
    );
    let mut errors = Vec::new();
    errors.extend(alice_send_result.unwrap_or_default());
    errors.extend(bob_send_result.unwrap_or_default());
    errors.extend(alice_recv_result.unwrap_or_default());
    errors.extend(bob_recv_result.unwrap_or_default());

    let num_secret_reuse_errors = errors
        .iter()
        .filter(|e| {
            e.contains("SecretReuseError")
                || e.contains("forward secrecy")
                || e.contains("out of bounds")
        })
        .count();
    println!(
        "Encountered {} errors, {} of which were SecretReuseError or forward secrecy errors.",
        errors.len(),
        num_secret_reuse_errors
    );
    assert_eq!(
        num_secret_reuse_errors, 0,
        "Expected no SecretReuseErrors but found: {:?}",
        num_secret_reuse_errors
    );
}
