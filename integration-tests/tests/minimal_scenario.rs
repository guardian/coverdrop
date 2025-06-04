use std::time::Duration;

use tokio::time;

use client::commands::journalist::messages::send_journalist_to_user_cover_message;
use integration_tests::{
    api_wrappers::{get_and_verify_public_keys, get_journalist_dead_drops, get_user_dead_drops},
    dev_j2u_mixing_config, dev_u2j_mixing_config, save_test_vector,
    utils::send_user_to_journalist_cover_messages,
    CoverDropStack,
};

/// This test is a minimal integration test that generates valid looking data for all public API
/// end-points. Since all messages are cover messages, no interesting communication happens.
#[tokio::test]
#[allow(clippy::await_holding_refcell_ref)]
async fn minimal_scenario() {
    pretty_env_logger::try_init().unwrap();

    let stack = CoverDropStack::builder().build().await;

    let anchor_org_pks = stack.keys().anchor_org_pks();

    let keys_and_profiles =
        get_and_verify_public_keys(stack.api_client_cached(), &anchor_org_pks, stack.now()).await;

    // User sends enough cover messages to trigger mixing two times
    // i.e. the journalist-facing dead drop should have two elements
    for _ in 0..2 {
        send_user_to_journalist_cover_messages(
            stack.messaging_client(),
            &keys_and_profiles.keys,
            dev_u2j_mixing_config().threshold_max,
        )
        .await;
        time::sleep(Duration::from_secs(1)).await; // just to get a different timestamp
    }

    // Journalist sends enough cover messages to trigger mixing three times
    // i.e. the user-facing dead drop should have three elements
    for _ in 0..3 {
        for _ in 0..dev_j2u_mixing_config().threshold_max {
            send_journalist_to_user_cover_message(stack.kinesis_client(), &keys_and_profiles.keys)
                .await
                .expect("Send journalist cover message");
        }
        time::sleep(Duration::from_secs(1)).await; // just to get a different timestamp
    }

    // Allow for CoverNode and API to process anything remaining
    time::sleep(Duration::from_secs(5)).await;

    let user_dead_drops = get_user_dead_drops(stack.api_client_cached(), 0).await;
    assert_eq!(user_dead_drops.len(), 3);

    let journalist_dead_drops = get_journalist_dead_drops(stack.api_client_cached(), 0).await;
    assert_eq!(journalist_dead_drops.len(), 2);

    assert!(!stack.do_secrets_exist_in_stack().await);
    // Only emitting one snapshot at the end to avoid confusion and keep the output minimal
    save_test_vector!("default", &stack);
}
