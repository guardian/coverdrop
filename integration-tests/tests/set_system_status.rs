use common::api::models::general::{StatusEvent, SystemStatus};
use integration_tests::{save_test_vector, CoverDropStack};

/// This tests that we have the correct initial state when we create a stack, and that
/// adding journalists works as expected.
///
/// Additionally it also checks that journalist keys are correctly verified and expired.
#[tokio::test]
async fn set_system_status() {
    pretty_env_logger::try_init().unwrap();

    let stack = CoverDropStack::new().await;

    // Get the initial status of the system
    {
        let initial_status = stack
            .api_client_cached()
            .get_latest_status()
            .await
            .expect("Get system status");

        assert_eq!(initial_status.status, SystemStatus::NoInformation);
        assert!(!initial_status.is_available);

        save_test_vector!("initial_status", &stack);
    }

    // Post a status update
    {
        let status_timestamp = stack.now();
        let new_event = StatusEvent::new(
            SystemStatus::Available,
            "All good!".into(),
            status_timestamp,
        );

        stack
            .api_client_cached()
            .post_status_event(new_event, &stack.keys().admin_key_pair, stack.now())
            .await
            .expect("Post system status");

        let status = stack
            .api_client_cached()
            .get_latest_status()
            .await
            .expect("Get system status");

        assert_eq!(status.status, SystemStatus::Available);
        assert!(status.is_available);

        save_test_vector!("status_available", &stack);
    }

    // Post another status update
    {
        let status_timestamp = stack.now();
        let new_event = StatusEvent::new(
            SystemStatus::Unavailable,
            "CoverDrop is currently unavailable. We are working on a fix.".into(),
            status_timestamp,
        );

        stack
            .api_client_cached()
            .post_status_event(new_event, &stack.keys().admin_key_pair, stack.now())
            .await
            .expect("Set system status to unavailable");

        let status = stack
            .api_client_cached()
            .get_latest_status()
            .await
            .expect("Get system status");

        assert_eq!(status.status, SystemStatus::Unavailable);
        assert!(!status.is_available);

        save_test_vector!("status_unavailable", &stack);
    }
}
