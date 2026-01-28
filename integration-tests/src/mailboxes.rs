use std::{
    cell::{RefCell, RefMut},
    path::Path,
};

use admin::generate_journalist;
use chrono::{DateTime, Utc};
use common::{
    api::{api_client::ApiClient, models::journalist_id::JournalistIdentity},
    client::{mailbox::user_mailbox::UserMailbox, JournalistStatus},
    crypto::keys::{serde::StorableKeyMaterial, signing::traits},
    protocol::{
        keys::{
            load_anchor_org_pks, AnchorOrganizationPublicKey, JournalistIdKeyPair,
            JournalistMessagingKeyPair, UntrustedJournalistIdKeyPair,
            UntrustedJournalistMessagingKeyPair, UntrustedUserKeyPair, UserKeyPair,
        },
        roles::{JournalistId, JournalistProvisioning},
    },
};
use journalist_vault::{JournalistVault, VAULT_EXTENSION};
use tempfile::TempDir;

use crate::secrets::MAILBOX_PASSWORD;

pub struct StackMailboxes {
    user_mailbox: RefCell<UserMailbox>,
    journalist_vault: JournalistVault,
    additional_journalist_vaults: Vec<JournalistVault>,
}

impl StackMailboxes {
    pub fn journalist(&self) -> JournalistVault {
        self.journalist_vault.clone()
    }

    pub fn additional_journalist(&self, index: usize) -> JournalistVault {
        self.additional_journalist_vaults
            .get(index)
            .expect("Get additional journalist, you may have not added additional journalist in the builder")
            .clone()
    }

    pub fn user(&self) -> RefMut<'_, UserMailbox> {
        self.user_mailbox.borrow_mut()
    }
}

/// Load the static keys for the integration tests. Useful when you want to output
/// consistent test vectors.
pub async fn load_mailboxes(
    api_client: &ApiClient,
    keys_path: impl AsRef<Path>,
    temp_dir: &TempDir,
    additional_journalists: u8,
    user_key_pair: &UserKeyPair,
    keys_generated_at: DateTime<Utc>,
    trust_anchors: Vec<AnchorOrganizationPublicKey>,
) -> StackMailboxes {
    //
    // Load the fixed vault using statically provided keys
    //

    let journalist_id = JournalistIdentity::new("static_test_journalist").unwrap();

    let journalist_vault = create_journalist_vault(
        journalist_id,
        "Static Test Journalist".to_string(),
        "journalist static test".to_string(),
        "static test journalist".to_string(),
        api_client,
        temp_dir,
        &keys_path,
        keys_generated_at,
        trust_anchors.clone(),
    )
    .await;

    //
    // Load any additional vaults using randomly generated keys
    //

    let mut additional_journalist_vaults: Vec<JournalistVault> = vec![];

    for index in 1..=additional_journalists {
        let journalist_id =
            JournalistIdentity::new(&format!("additional_test_journalist_{index}")).unwrap();
        let display_name = format!("Additional Test Journalist {index}");
        let sort_name = format!("journalist additional test {index}");
        let description = format!("Additional Test Journalist {index}");

        let vault = create_journalist_vault(
            journalist_id,
            display_name,
            sort_name,
            description,
            api_client,
            temp_dir,
            &keys_path,
            keys_generated_at,
            trust_anchors.clone(),
        )
        .await;
        additional_journalist_vaults.push(vault);
    }

    // Wait for the journalists to be picked up by the CoverNode
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let tofu_org_pks = load_anchor_org_pks(&keys_path, keys_generated_at).expect("Load org pks");

    let user_mailbox = UserMailbox::new_with_keys(
        MAILBOX_PASSWORD,
        user_key_pair.clone(),
        tofu_org_pks,
        temp_dir.path(),
    )
    .expect("Create user mailbox");

    StackMailboxes {
        user_mailbox: RefCell::new(user_mailbox),
        journalist_vault,
        additional_journalist_vaults,
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn create_journalist_vault(
    journalist_id: JournalistIdentity,
    display_name: String,
    sort_name: String,
    description: String,
    api_client: &ApiClient,
    temp_dir: &TempDir,
    keys_path: impl AsRef<Path>,
    keys_generated_at: DateTime<Utc>,
    trust_anchors: Vec<AnchorOrganizationPublicKey>,
) -> JournalistVault {
    let vault_path = temp_dir
        .path()
        .join(journalist_id.as_ref())
        .with_extension(VAULT_EXTENSION);

    generate_journalist(
        keys_path,
        display_name,
        None,
        Some(sort_name),
        description,
        false, // is_desk
        MAILBOX_PASSWORD,
        JournalistStatus::Visible,
        &vault_path,
        keys_generated_at,
        trust_anchors.clone(),
    )
    .await
    .expect("Generate vault");

    let vault = JournalistVault::open(&vault_path, MAILBOX_PASSWORD, trust_anchors)
        .await
        .expect("Load desk vault");

    vault
        .process_vault_setup_bundle(api_client, keys_generated_at)
        .await
        .expect("Onboard vault");

    vault
}

// Functions for loading user keys directly from the disk
// Normally we never store user keys on disk, but for testing we need to be able to
// load them into the mailboxes we create every time.
// Note this only supports a single key pair for journalist msg and ID roles per directory.
pub fn load_journalist_id_key_pair(
    keys_path: impl AsRef<Path>,
    journalist_provisioning_pk: &impl traits::PublicSigningKey<JournalistProvisioning>,
    now: DateTime<Utc>,
) -> JournalistIdKeyPair {
    UntrustedJournalistIdKeyPair::load_from_directory(&keys_path)
        .expect("Should load journalist ID key pair")[0]
        .to_trusted(journalist_provisioning_pk, now)
        .expect("Verify signing key pair")
}

pub fn load_journalist_msg_key_pair(
    keys_path: impl AsRef<Path>,
    journalist_id_pk: &impl traits::PublicSigningKey<JournalistId>,
    now: DateTime<Utc>,
) -> JournalistMessagingKeyPair {
    UntrustedJournalistMessagingKeyPair::load_from_directory(&keys_path)
        .expect("Should load journalist messaging key pair")[0]
        .to_trusted(journalist_id_pk, now)
        .expect("Verify journalist messaging key pair")
}

pub fn load_user_key_pair(keys_path: impl AsRef<Path>) -> UserKeyPair {
    UntrustedUserKeyPair::load_from_directory(&keys_path).expect("Load user key pairs")[0]
        .to_trusted()
}
