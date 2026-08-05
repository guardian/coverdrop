use chrono::Utc;
use common::protocol::keys::JournalistProvisioningPublicKey;
use common::{
    api::models::{journalist_id::JournalistIdentity, sentinel_id::SentinelIdentity},
    clap::Stage,
    protocol::keys::{generate_journalist_provisioning_key_pair, generate_organization_key_pair},
};
use journalist_vault::JournalistVault;
use std::path::{Path, PathBuf};
use tempfile::tempdir_in;
use trust_anchors::get_trust_anchors;

mod test_utils;

/// A vault created without a stage (simulating a pre-migration vault) should have its stage
/// backfilled on the first open and then enforce that stage on subsequent opens.
#[tokio::test]
async fn vault_without_stage_is_backfilled_on_open() {
    let (db_path, _tmp, journalist_id, provisioning_pks) = setup();
    let now = Utc::now();

    let trust_anchors = get_trust_anchors(&Stage::Development, now).expect("loaded trust anchors");

    // Create a vault *without* a stage, simulating a pre-migration vault.
    JournalistVault::create_without_stage(
        &db_path,
        test_utils::TEST_PASSPHRASE_VAULT,
        &journalist_id,
        &SentinelIdentity::new("test_sentinel").unwrap(),
        &provisioning_pks,
        now,
        trust_anchors,
    )
    .await
    .expect("Create vault without stage");

    // First open: stage is None so it gets backfilled with Development.
    let vault = open(&db_path, Stage::Development)
        .await
        .expect("First open should backfill the stage");
    assert_eq!(journalist_id, vault.journalist_id().await.unwrap());

    // Second open with same stage: succeeds because it was backfilled.
    open(&db_path, Stage::Development)
        .await
        .expect("Second open with matching stage should succeed");

    // Third open with different stage: fails.
    assert_open_fails_with_stage_mismatch(&db_path, Stage::Staging).await;
}

/// A vault created *with* a stage rejects opens using a different stage from the start.
#[tokio::test]
async fn vault_with_stage_rejects_wrong_stage() {
    let (db_path, _tmp, journalist_id, provisioning_pks) = setup();
    let now = Utc::now();

    JournalistVault::create(
        &db_path,
        test_utils::TEST_PASSPHRASE_VAULT,
        &journalist_id,
        &SentinelIdentity::new("test_sentinel").unwrap(),
        &provisioning_pks,
        now,
        Stage::Development,
    )
    .await
    .expect("Create vault with Development stage");

    open(&db_path, Stage::Development)
        .await
        .expect("Open with matching stage should succeed");

    assert_open_fails_with_stage_mismatch(&db_path, Stage::Staging).await;
}

fn setup() -> (
    PathBuf,
    tempfile::TempDir,
    JournalistIdentity,
    Vec<JournalistProvisioningPublicKey>,
) {
    let tmp = tempdir_in(std::env::current_dir().unwrap()).unwrap();
    let db_path = tmp.path().join("test.db");
    let now = Utc::now();
    let journalist_id = JournalistIdentity::new("stage-test-journalist").unwrap();
    let org_key_pair = generate_organization_key_pair(now);
    let kp = generate_journalist_provisioning_key_pair(&org_key_pair, now);
    (db_path, tmp, journalist_id, vec![kp.public_key().clone()])
}

async fn open(
    path: &Path,
    stage: Stage,
) -> Result<JournalistVault, journalist_vault::OpenVaultError> {
    JournalistVault::open(path, test_utils::TEST_PASSPHRASE_VAULT, stage).await
}

async fn assert_open_fails_with_stage_mismatch(path: &Path, stage: Stage) {
    let result = open(path, stage).await;
    let err = match result {
        Err(e) => e,
        Ok(_) => panic!("Opening with wrong stage should fail"),
    };
    assert!(
        err.to_string().contains("stage"),
        "Error should mention stage mismatch, got: {err}"
    );
}
