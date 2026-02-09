use std::vec;

use chrono::Utc;
use common::{
    api::models::journalist_id::JournalistIdentity,
    protocol::keys::{generate_journalist_provisioning_key_pair, generate_organization_key_pair},
};
use journalist_vault::JournalistVault;
use tempfile::tempdir_in;

#[tokio::test]
async fn sync_provisioning_keys_to_vault() {
    let temp_dir = tempdir_in(std::env::current_dir().unwrap()).unwrap();
    let mut db_path = temp_dir.path().to_owned();
    db_path.push("test.db");

    let now = Utc::now();

    let journalist_id = JournalistIdentity::new("Hello").unwrap();

    let org_key_pair = generate_organization_key_pair(now);
    let trust_anchors = vec![org_key_pair.public_key().clone().into_anchor()];

    let journalist_provisioning_key_pair_1 =
        generate_journalist_provisioning_key_pair(&org_key_pair, now);

    // Setup - create vault
    {
        let journalist_provisioning_pks =
            vec![journalist_provisioning_key_pair_1.public_key().clone()];

        let _ = JournalistVault::create(
            &db_path,
            "test_password",
            &journalist_id,
            &journalist_provisioning_pks,
            now,
            trust_anchors.clone(),
        )
        .await
        .expect("Create journalist vault");
    }

    // Open vault with correct password
    let vault = JournalistVault::open(&db_path, "test_password", trust_anchors.clone())
        .await
        .expect("Load journalist vault");
    let vault_journalist_id = vault.journalist_id().await.expect("Get journalist ID");
    assert_eq!(journalist_id, vault_journalist_id);

    // after initial set up there is one provisioning pk in the vault.
    let vault_journalist_provisioning_pks = vault
        .provisioning_pks(now)
        .await
        .expect("got provisioning pks from database");
    assert_eq!(vault_journalist_provisioning_pks.len(), 1);

    // Success case: create a new provisioning key signed by the existing trust anchor.
    // After syncing the list of two keys to the database, there should be two provisioning keys in the database,
    // the original key and the new key.
    let journalist_provisioning_key_pair_2 =
        generate_journalist_provisioning_key_pair(&org_key_pair, now);

    let original_provisioning_and_valid_provisioning = vec![
        journalist_provisioning_key_pair_1.public_key(),
        journalist_provisioning_key_pair_2.public_key(),
    ];
    vault
        .sync_journalist_provisioning_pks(&original_provisioning_and_valid_provisioning, now)
        .await
        .expect("call to sync_public_keys successful");

    let vault_journalist_provisioning_pks = vault
        .provisioning_pks(now)
        .await
        .expect("got provisioning pks from database");
    assert_eq!(vault_journalist_provisioning_pks.len(), 2);
    assert!(
        vault_journalist_provisioning_pks.contains(journalist_provisioning_key_pair_1.public_key()),
    );
    assert!(
        vault_journalist_provisioning_pks.contains(journalist_provisioning_key_pair_2.public_key()),
    );

    // Attempt to sync a list of provision keys containing a key signed by an org key that does
    // not exist in the vault.
    let org_key_pair_2 = generate_organization_key_pair(now);
    let journalist_provisioning_key_pair_3 =
        generate_journalist_provisioning_key_pair(&org_key_pair_2, now);

    let original_provisioning_and_invalid_provisioning =
        vec![journalist_provisioning_key_pair_3.public_key()];
    vault
        .sync_journalist_provisioning_pks(&original_provisioning_and_invalid_provisioning, now)
        .await
        .expect("call to sync_public_keys successful");

    let vault_journalist_provisioning_pks = vault
        .provisioning_pks(now)
        .await
        .expect("got provisioning pks from database");

    // Only the first two keys exist in the vault
    // The third, invalid provisioning key was not inserted.
    assert_eq!(vault_journalist_provisioning_pks.len(), 2);
    assert!(
        vault_journalist_provisioning_pks.contains(journalist_provisioning_key_pair_1.public_key()),
    );
    assert!(
        vault_journalist_provisioning_pks.contains(journalist_provisioning_key_pair_2.public_key()),
    );
}
