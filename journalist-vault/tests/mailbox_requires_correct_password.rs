use chrono::Utc;
use common::{
    api::models::{journalist_id::JournalistIdentity, sentinel_id::SentinelIdentity},
    clap::Stage,
    protocol::keys::{generate_journalist_provisioning_key_pair, generate_organization_key_pair},
};
use journalist_vault::JournalistVault;
use tempfile::tempdir_in;

mod test_utils;

#[tokio::test]
async fn vault_requires_correct_password() {
    let temp_dir = tempdir_in(std::env::current_dir().unwrap()).unwrap();
    let mut db_path = temp_dir.path().to_owned();
    db_path.push("test.db");

    let now = Utc::now();

    let journalist_id = JournalistIdentity::new("Hello").unwrap();
    let org_key_pair = generate_organization_key_pair(now);

    let journalist_provisioning_key_pair =
        generate_journalist_provisioning_key_pair(&org_key_pair, now);

    // Create vault
    {
        let journalist_provisioning_pks =
            vec![journalist_provisioning_key_pair.public_key().clone()];

        let _ = JournalistVault::create(
            &db_path,
            test_utils::TEST_PASSPHRASE_VAULT,
            &journalist_id,
            &SentinelIdentity::new("test_sentinel").unwrap(),
            &journalist_provisioning_pks,
            now,
            Stage::Development,
        )
        .await
        .expect("Create journalist vault");
    }

    // Open vault with correct password
    {
        let vault = JournalistVault::open(
            &db_path,
            test_utils::TEST_PASSPHRASE_VAULT,
            Stage::Development,
        )
        .await
        .expect("Load journalist vault");

        let vault_journalist_id = vault.journalist_id().await.expect("Get journalist ID");

        assert_eq!(journalist_id, vault_journalist_id);
    }

    // Open vault with wrong password
    {
        let vault =
            JournalistVault::open(&db_path, "wrong_test_password", Stage::Development).await;
        assert!(vault.is_err());
    }
}
