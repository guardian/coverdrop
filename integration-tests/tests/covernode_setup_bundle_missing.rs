use integration_tests::keys::CoverNodeKeyMode;
use integration_tests::CoverDropStack;

#[tokio::test]
#[should_panic]
async fn covernode_setup_bundle_missing_test() {
    // if neither the covernode_setup_bundle nor the covernode_id_key are available, the CoverDropStack should panic
    // this happens in publish_keys_task
    CoverDropStack::builder()
        .with_covernode_key_mode(CoverNodeKeyMode::NoSetup)
        .build()
        .await;
}
