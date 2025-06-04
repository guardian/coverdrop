use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Utc};
use common::{
    api::{
        api_client::ApiClient,
        forms::{
            PostAdminPublicKeyForm, PostCoverNodeIdPublicKeyForm,
            PostCoverNodeProvisioningPublicKeyForm, PostJournalistProvisioningPublicKeyForm,
        },
        models::covernode_id::CoverNodeIdentity,
    },
    crypto::keys::serde::set_key_permissions,
    protocol::keys::{
        load_anchor_org_pks, load_covernode_id_key_pairs, load_covernode_msg_key_pairs,
        load_covernode_provisioning_key_pairs, load_journalist_provisioning_key_pairs,
        load_org_key_pairs, AnchorOrganizationPublicKey, CoverNodeIdKeyPair,
        CoverNodeMessagingKeyPair, CoverNodeProvisioningKeyPair, JournalistIdKeyPair,
        JournalistMessagingKeyPair, JournalistProvisioningKeyPair, LatestKey, OrganizationKeyPair,
        UserKeyPair,
    },
    system::keys::{load_admin_key_pair, AdminKeyPair},
};

use crate::constants::COVERNODE_DB_PASSWORD;
use crate::mailboxes::{
    load_journalist_id_key_pair, load_journalist_msg_key_pair, load_user_key_pair,
};
use covernode_database::Database;

pub fn get_keys_generated_at_time(keys_path: impl AsRef<Path>) -> DateTime<Utc> {
    let mut timestamp_path = keys_path.as_ref().to_owned();
    timestamp_path.push("keys_generated_at.txt");
    DateTime::parse_from_rfc3339(
        fs::read_to_string(timestamp_path)
            .expect("Read keys_generated_at.txt file")
            .trim(),
    )
    .expect("Parse keys_generated_at.txt timestamp")
    .with_timezone(&Utc)
}

pub struct StackKeys {
    pub keys_generated_at: DateTime<Utc>,
    pub org_key_pair: OrganizationKeyPair,
    pub covernode_provisioning_key_pair: CoverNodeProvisioningKeyPair,
    pub covernode_id_key_pair: CoverNodeIdKeyPair,
    pub covernode_msg_key_pair: CoverNodeMessagingKeyPair,
    pub journalist_provisioning_key_pair: JournalistProvisioningKeyPair,
    pub journalist_id_key_pair: JournalistIdKeyPair,
    pub journalist_msg_key_pair: JournalistMessagingKeyPair,
    pub admin_key_pair: AdminKeyPair,
    pub user_key_pair: UserKeyPair,
}

impl StackKeys {
    // A small hack to allow conversion from unverified to verified key
    // hierarchies - could improve this so that the test clients
    // don't have full knowledge of all the keys.
    pub fn anchor_org_pks(&self) -> Vec<AnchorOrganizationPublicKey> {
        vec![self.org_key_pair.public_key().clone().into_anchor()]
    }
}

pub fn get_static_keys_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("keys")
}

pub fn ensure_key_permissions() {
    let keys_path = get_static_keys_path();

    // Set permissions - this works around a limitation in git that means
    // we can't maintain the file permissions through checkouts
    let json_ext = Some(OsStr::new("json"));
    fs::read_dir(keys_path)
        .expect("Read keys path")
        .filter_map(|entry| {
            let entry = entry.unwrap();
            if entry.path().is_file() && entry.path().extension() == json_ext {
                Some(entry)
            } else {
                None
            }
        })
        .for_each(|entry| {
            let path = entry.path();
            set_key_permissions(path);
        });
}

/// Load the static keys for the integration tests. Useful when you want to output
/// consistent test vectors.
pub fn load_static_stack_keys(now: DateTime<Utc>) -> StackKeys {
    let keys_path = get_static_keys_path();

    // Load the various key pairs
    let anchor_org_pks =
        load_anchor_org_pks(&keys_path, now).expect("Load trusted org public keys");

    let mut org_key_pairs = load_org_key_pairs(&keys_path, now).expect("Load org key pair");

    let mut covernode_provisioning_key_pairs =
        load_covernode_provisioning_key_pairs(&keys_path, &anchor_org_pks, now)
            .expect("Load covernode provisioning key pair");

    let mut covernode_id_key_pairs =
        load_covernode_id_key_pairs(&keys_path, &covernode_provisioning_key_pairs, now)
            .expect("Load covernode identity key");

    let mut covernode_msg_key_pairs =
        load_covernode_msg_key_pairs(&keys_path, &covernode_id_key_pairs, now)
            .expect("Load covernode messaging keys");

    let mut journalist_provisioning_key_pairs =
        load_journalist_provisioning_key_pairs(&keys_path, &anchor_org_pks, now)
            .expect("Load journalist provisioning key pair");

    let journalist_id_key_pair = load_journalist_id_key_pair(
        &keys_path,
        journalist_provisioning_key_pairs.first().unwrap(),
        now,
    );

    let journalist_msg_key_pair =
        load_journalist_msg_key_pair(&keys_path, &journalist_id_key_pair, now);

    let admin_key_pair = load_admin_key_pair(&keys_path, &anchor_org_pks, now)
        .expect("Load system status key pair")
        .into_latest_key()
        .expect("Get latest system status key");

    let org_key_pair = org_key_pairs.remove(0);
    let covernode_provisioning_key_pair = covernode_provisioning_key_pairs.remove(0);
    let covernode_id_key_pair = covernode_id_key_pairs.remove(0);
    let covernode_msg_key_pair = covernode_msg_key_pairs.remove(0);
    let journalist_provisioning_key_pair = journalist_provisioning_key_pairs.remove(0);

    let user_key_pair = load_user_key_pair(&keys_path);

    StackKeys {
        keys_generated_at: now,
        org_key_pair,
        covernode_provisioning_key_pair,
        covernode_id_key_pair,
        covernode_msg_key_pair,
        journalist_provisioning_key_pair,
        journalist_id_key_pair,
        journalist_msg_key_pair,
        admin_key_pair,
        user_key_pair,
    }
}

pub enum CoverNodeKeyMode {
    /// The test CoverNode will use a setup bundle to bootstrap itself.
    SetupBundle,
    /// The test CoverNode will run using keys provided by the test infrastructure. This is faster than using
    /// the setup bundle so is preferred unless you're testing the bundle.
    ProvidedKeyPair,
    /// Use neither a provided key pair or setup bundle. This is only useful for testing error states.
    NoSetup,
}

pub async fn open_covernode_database(
    covernode_keys_dir: &Path,
    covernode_id: &CoverNodeIdentity,
) -> anyhow::Result<Database> {
    let db_path = format!("{}/{}.db", covernode_keys_dir.display(), covernode_id);

    let db = Database::open(&db_path, COVERNODE_DB_PASSWORD)
        .await
        .expect("Open connection with database");

    Ok(db)
}

pub async fn add_stack_keys_to_api(
    keys: &StackKeys,
    api_client: &ApiClient,
    now: DateTime<Utc>,
    covernode_database: Database,
    covernode_key_mode: CoverNodeKeyMode,
) {
    let StackKeys {
        org_key_pair,
        covernode_provisioning_key_pair,
        covernode_id_key_pair,
        covernode_msg_key_pair,
        journalist_provisioning_key_pair,
        admin_key_pair,
        ..
    } = keys;

    let form = PostCoverNodeProvisioningPublicKeyForm::new(
        covernode_provisioning_key_pair.public_key().to_untrusted(),
        org_key_pair,
        now,
    )
    .expect("Create CoverNode provisioning public key upload form");

    api_client
        .post_covernode_provisioning_pk(form)
        .await
        .expect("Upload CoverNode provisioning public key");

    let covernode_identity =
        &CoverNodeIdentity::new("covernode_001").expect("Make covernode identity");

    match covernode_key_mode {
        CoverNodeKeyMode::SetupBundle => {
            // if the setup bundle exists, that's all we need - we don't need to publish/insert the keys
            let form = &PostCoverNodeIdPublicKeyForm::new(
                covernode_identity.clone(),
                covernode_id_key_pair.public_key().to_untrusted(),
                covernode_provisioning_key_pair,
                now,
            )
            .expect("Create covernode id public key form");

            covernode_database
                .insert_setup_bundle(form, covernode_id_key_pair, now)
                .await
                .expect("Insert setup bundle");
        }
        CoverNodeKeyMode::ProvidedKeyPair => {
            // To speed up tests, here we manually publish the covernode id/msg key pairs and insert them
            // into the covernode database. When we're actually trying to test the setup bundle process then
            // we don't want to do this
            let id_epoch = api_client
                .post_covernode_id_pk(
                    covernode_identity,
                    covernode_id_key_pair.public_key(),
                    covernode_provisioning_key_pair,
                    now,
                )
                .await
                .expect("Upload CoverNode ID public key");

            covernode_database
                .insert_id_key_pair_with_epoch(covernode_id_key_pair, id_epoch, now)
                .await
                .expect("Insert covernode id key pair");

            let msg_epoch = api_client
                .post_covernode_msg_pk(
                    covernode_msg_key_pair.public_key(),
                    covernode_id_key_pair,
                    now,
                )
                .await
                .expect("Upload CoverNode messaging public key");

            covernode_database
                .insert_msg_key_pair_add_epoch(covernode_msg_key_pair, msg_epoch, now)
                .await
                .expect("Insert message key pair");
        }
        CoverNodeKeyMode::NoSetup => {
            tracing::warn!("Neither setup bundle nor key pair available, can't insert setup bundle or publish id/msg key pairs - covernode will panic")
        }
    }

    let form = PostJournalistProvisioningPublicKeyForm::new(
        journalist_provisioning_key_pair.public_key().to_untrusted(),
        org_key_pair,
        now,
    )
    .expect("Create journalist provisioning public key upload form");

    api_client
        .post_journalist_provisioning_pk(form)
        .await
        .expect("Upload journalist provisioning public key");

    let form = PostAdminPublicKeyForm::new(
        admin_key_pair.public_key().to_untrusted(),
        org_key_pair,
        now,
    )
    .expect("Create system status public key upload form");

    api_client
        .post_admin_pk(form)
        .await
        .expect("upload system status public key");
}
