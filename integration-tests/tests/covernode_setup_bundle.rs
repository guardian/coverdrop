use std::time::Duration;

use integration_tests::keys::CoverNodeKeyMode;
use integration_tests::CoverDropStack;
use tokio::time::sleep;

#[tokio::test]
#[allow(clippy::await_holding_refcell_ref)]
async fn covernode_setup_bundle_test() -> anyhow::Result<()> {
    pretty_env_logger::try_init().unwrap();

    let stack = CoverDropStack::builder()
        .with_covernode_key_mode(CoverNodeKeyMode::SetupBundle)
        .build()
        .await;

    // Wait a short while for the CoverNode to publish its set up bundle
    sleep(Duration::from_secs(5)).await;

    let db = stack.covernode_database();

    let setup_bundle = db.select_setup_bundle().await.expect("setup bundle query");

    // after the covernode has published the setup bundle to the api it should clear the setup_bundle
    // table - let's check that has happened
    assert!(setup_bundle.is_none());

    let id_key = db
        .select_published_id_key_pairs()
        .await
        .expect("Get published id key pairs");

    assert_eq!(id_key.len(), 1);

    Ok(())
}
