//! This module contains wrappers for API functions which reduce the amount of noise
//! in integration tests by simply panicking if anything goes wrong.

use std::path::Path;

use admin::generate_journalist;
use chrono::{DateTime, Utc};
use common::{
    api::{
        api_client::ApiClient,
        models::{
            dead_drops::{
                DeadDropId, UnverifiedJournalistToUserDeadDropsList,
                UnverifiedUserToJournalistDeadDropsList,
            },
            general::PublishedStatusEvent,
            untrusted_keys_and_journalist_profiles::UntrustedKeysAndJournalistProfiles,
        },
    },
    client::{JournalistStatus, VerifiedKeysAndJournalistProfiles},
    protocol::keys::AnchorOrganizationPublicKey,
    service::{self, CoverNode, IdentityApi},
    task::TaskApiClient,
};
use journalist_vault::JournalistVault;

use crate::secrets::MAILBOX_PASSWORD;

//
// Public keys
//

pub async fn get_public_keys(api_client: &ApiClient) -> UntrustedKeysAndJournalistProfiles {
    api_client
        .get_public_keys()
        .await
        .inspect_err(|e| {
            eprintln!(
                "Inner error: {:?}",
                e.downcast_ref::<reqwest::Error>().unwrap()
            );
        })
        .expect("Get public keys")
}

pub async fn get_and_verify_public_keys(
    api_client: &ApiClient,
    anchor_org_pks: &[AnchorOrganizationPublicKey],
    now: DateTime<Utc>,
) -> VerifiedKeysAndJournalistProfiles {
    get_public_keys(api_client)
        .await
        .into_trusted(anchor_org_pks, now)
}

pub async fn generate_test_journalist(
    api_client: &ApiClient,
    keys_dir: impl AsRef<Path>,
    vault_path: impl AsRef<Path>,
    now: DateTime<Utc>,
    trust_anchors: Vec<AnchorOrganizationPublicKey>,
) {
    generate_journalist(
        keys_dir,
        "Generated Test Journalist".into(),
        None,
        Some("journalist generated test".into()),
        "This is a test journalist".into(),
        false,
        MAILBOX_PASSWORD,
        JournalistStatus::Visible,
        &vault_path,
        now,
        trust_anchors.clone(),
    )
    .await
    .expect("Create journalist");

    let vault_path = vault_path.as_ref().join("generated_test_journalist.vault");

    let vault = JournalistVault::open(&vault_path, MAILBOX_PASSWORD, trust_anchors)
        .await
        .expect("Load desk vault");

    vault
        .process_vault_setup_bundle(api_client, now)
        .await
        .expect("Onboard vault");
}

pub async fn generate_test_desk(
    api_client: &ApiClient,
    keys_dir: impl AsRef<Path>,
    vault_path: impl AsRef<Path>,
    now: DateTime<Utc>,
    trust_anchors: Vec<AnchorOrganizationPublicKey>,
) {
    generate_journalist(
        keys_dir,
        "Generated Test Desk".into(),
        None,
        Some("desk generated test".into()),
        "This is a test desk".into(),
        true,
        MAILBOX_PASSWORD,
        JournalistStatus::Visible,
        &vault_path,
        now,
        trust_anchors.clone(),
    )
    .await
    .expect("Create desk");

    let desk_vault_path = vault_path.as_ref().join("generated_test_desk.vault");

    let desk_vault = JournalistVault::open(&desk_vault_path, MAILBOX_PASSWORD, trust_anchors)
        .await
        .expect("Load desk vault");

    desk_vault
        .process_vault_setup_bundle(api_client, now)
        .await
        .expect("Onboard vault");
}

pub async fn upload_new_messaging_key(
    api_client: &ApiClient,
    vault: &JournalistVault,
    now: DateTime<Utc>,
) {
    vault
        .generate_msg_key_pair_and_upload_pk(api_client, now)
        .await
        .expect("Create and upload journalist messaging key")
}

//
// Dead drops
//

pub async fn get_journalist_dead_drops(
    api_client: &ApiClient,
    ids_greater_than: DeadDropId,
) -> UnverifiedUserToJournalistDeadDropsList {
    api_client
        .pull_all_journalist_dead_drops(ids_greater_than)
        .await
        .expect("Get journalist dead drops")
}

pub async fn get_user_dead_drops(
    api_client: &ApiClient,
    ids_greater_than: DeadDropId,
) -> UnverifiedJournalistToUserDeadDropsList {
    api_client
        .pull_user_dead_drops(ids_greater_than)
        .await
        .expect("Get journalist dead drops")
}

//
// System status
//

pub async fn get_latest_status(api_client: &ApiClient) -> PublishedStatusEvent {
    api_client
        .get_latest_status()
        .await
        .expect("Get latest system status")
}

//
// Task Runner APIs
//

pub async fn trigger_key_rotation_covernode(task_api_client: &TaskApiClient<CoverNode>) {
    task_api_client
        .trigger_task("create_keys")
        .await
        .expect("Trigger key rotation task");

    task_api_client
        .trigger_task("publish_keys")
        .await
        .expect("Trigger key publishing task");
}

pub async fn trigger_all_init_tasks_covernode(task_api_client: &TaskApiClient<CoverNode>) {
    task_api_client
        .trigger_task("refresh_tag_look_up_table")
        .await
        .expect("Trigger refresh tag look up table task");

    task_api_client
        .trigger_task("publish_keys")
        .await
        .expect("Trigger publish keys task");

    task_api_client
        .trigger_task("create_keys")
        .await
        .expect("Trigger create keys task");
}

pub async fn trigger_expired_key_deletion_covernode(task_api_client: &TaskApiClient<CoverNode>) {
    task_api_client
        .trigger_task("delete_expired_keys")
        .await
        .expect("Trigger key rotation task");
}

pub async fn trigger_expired_key_deletion_identity_api(
    task_api_client: &TaskApiClient<IdentityApi>,
) {
    task_api_client
        .trigger_task("delete_expired_keys")
        .await
        .expect("Trigger key rotation task");
}

pub async fn trigger_load_org_pk_api(task_api_client: &TaskApiClient<service::Api>) {
    task_api_client
        .trigger_task("anchor_org_pk_poll")
        .await
        .expect("Trigger org pk poll task");
}
