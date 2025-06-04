use common::crypto::keys::serde::StorableKeyMaterial;

use admin::{
    anchor_public_key_bundle::{save_anchor_public_key_bundle, AnchorOrganizationPublicKeyBundle},
    api_has_anchor_org_pk, copy_anchor_org_pk, read_bundle_from_disk,
    save_organization_key_pair_bundle, ANCHOR_ORGANIZATION_PUBLIC_KEY_BUNDLE_FILENAME,
};
use chrono::Duration;
use common::{
    protocol::{
        constants::ORGANIZATION_KEY_VALID_DURATION_SECONDS, keys::generate_organization_key_pair,
    },
    throttle::Throttle,
    time,
};
use core::time::Duration as CoreDuration;

use integration_tests::{
    api_wrappers::{get_journalist_dead_drops, get_user_dead_drops},
    CoverDropStack,
};

/// This tests that we have the correct initial state when we create a stack, and that
/// adding journalists works as expected.
///
/// Additionally it also checks that journalist keys are correctly verified and expired.
#[tokio::test]
#[allow(clippy::await_holding_refcell_ref)]
async fn vault_manager_test() -> anyhow::Result<()> {
    pretty_env_logger::try_init().unwrap();

    let mut stack = CoverDropStack::builder().build().await;

    //
    // Confirm clean initial state
    //

    let user_dead_drops = get_user_dead_drops(stack.api_client_cached(), 0).await;
    let journalist_dead_drops = get_journalist_dead_drops(stack.api_client_cached(), 0).await;
    assert!(user_dead_drops.is_empty());
    assert!(journalist_dead_drops.is_empty());

    // we want to test the reseeding, so we can check our error logging is ok
    // Setup the stack so that we have valid journalist provisioning, identity and messaging keys.
    // Then we time travel so that all the keys are expired
    // travel ORGANIZATION_KEY_VALID_DURATION_SECONDS + 1 day into the future, there should be no valid id key any more
    let future = stack.now()
        + Duration::seconds(ORGANIZATION_KEY_VALID_DURATION_SECONDS)
        + Duration::days(1);

    stack.time_travel(future).await;

    // Reseed the organization keys
    let org_key_pair = generate_organization_key_pair(stack.now());

    org_key_pair
        .to_untrusted()
        .save_to_disk(stack.temp_dir_path())?;

    // For the org key pair we need to store the public keys separately
    // to the secret keys since we must distribute them to clients
    // as trusted public keys.

    org_key_pair
        .public_key()
        .to_untrusted()
        .save_to_disk(stack.temp_dir_path())?;

    save_organization_key_pair_bundle(stack.temp_dir_path(), &org_key_pair)?;

    let anchor_org_pk = org_key_pair.public_key().clone().into_anchor();

    save_anchor_public_key_bundle(stack.temp_dir_path(), &anchor_org_pk)?;

    copy_anchor_org_pk(stack.temp_dir_path(), stack.api_keys_path(), stack.now()).await?;

    let base_path = stack.temp_dir_path();

    // Trusted org pk bundle
    let bundle = base_path.join(ANCHOR_ORGANIZATION_PUBLIC_KEY_BUNDLE_FILENAME);
    let anchor_org_pk_bundle = read_bundle_from_disk::<AnchorOrganizationPublicKeyBundle>(bundle)?;

    let started_polling = time::now();
    let max_duration = chrono::Duration::minutes(10);
    let max_duration_seconds = max_duration.num_seconds();
    let mut throttle = Throttle::new(CoreDuration::from_secs(10));

    while !api_has_anchor_org_pk(
        stack.api_client_cached(),
        &anchor_org_pk_bundle.anchor_org_pk,
    )
    .await?
    {
        let elapsed = time::now() - started_polling;

        println!(
            "Waiting for new organization key to appear in API (waited {}s/{}s)",
            elapsed.num_seconds(),
            max_duration_seconds
        );

        if elapsed > max_duration {
            anyhow::bail!(
                "Trusted organization key does not appear in API after {} seconds of checking",
                elapsed.num_seconds()
            );
        }

        throttle.wait().await;
    }

    assert!(!stack.do_secrets_exist_in_stack().await);

    Ok(())
}
