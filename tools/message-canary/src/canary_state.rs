use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Duration};

use chrono::{DateTime, Utc};
use common::{
    api::api_client::ApiClient,
    client::VerifiedKeysAndJournalistProfiles,
    protocol::keys::{load_anchor_org_pks, UserKeyPair},
    time,
    u2j_appender::messaging_client::MessagingClient,
};
use journalist_vault::JournalistVault;
use message_canary_database::{database::Database, model::User};
use tokio::{sync::RwLock, time::sleep};

/// Wraps the state used by the canary. This just reduces the amount of cloning we need
/// for creating each of the async tasks.
///
/// Caches opened vaults since performing the vault key derivation and migration is quite slow.
#[derive(Clone)]
pub struct CanaryState {
    pub api_client: ApiClient,
    pub messaging_client: MessagingClient,

    pub db: Database,
    pub keys_path: PathBuf,

    pub users: Vec<User>,

    // Wrapping the vault map in a arc and lock so it can be copied
    // and updated easily
    vaults: Arc<RwLock<HashMap<PathBuf, JournalistVault>>>,
}

impl CanaryState {
    pub async fn new(
        keys_path: impl Into<PathBuf>,
        vaults_path: impl Into<PathBuf>,
        api_client: ApiClient,
        messaging_client: MessagingClient,
        db: Database,
        num_users: u16,
    ) -> anyhow::Result<Self> {
        //
        // Vaults
        //
        let vaults_path = vaults_path.into();

        let mut vaults = HashMap::new();
        let mut paths = tokio::fs::read_dir(&vaults_path).await?;

        let mut any_new_journalists = false;
        while let Some(path) = paths.next_entry().await? {
            let path = path.path();
            if path.extension().is_some_and(|e| e == "vault") {
                let password_path: PathBuf = path.with_extension("password");

                if !password_path.exists() {
                    anyhow::bail!(
                        "Vault {} does not have matching password file",
                        path.display()
                    );
                }

                let password = std::fs::read_to_string(&password_path)?;

                let vault = JournalistVault::open(&path, &password).await?;

                let set_up_occured = vault
                    .process_vault_setup_bundle(&api_client, time::now())
                    .await?;

                if set_up_occured {
                    any_new_journalists = true;
                }

                let journalist_id = vault.journalist_id().await?;

                vaults.insert(path.to_owned(), vault);
                db.insert_journalist(&journalist_id).await?;
            }
        }

        if any_new_journalists {
            tracing::info!(
                "New journalists added to message canary, sleeping until they appear in the API"
            );
            // This should come from a shared crate
            // https://github.com/guardian/coverdrop/issues/2784
            sleep(Duration::from_secs(60)).await;
        }

        //
        // Users
        //

        let users = db.get_users(num_users).await?;

        let missing_users = (num_users as usize).saturating_sub(users.len());

        for _ in 0..missing_users {
            let key_pair = UserKeyPair::generate();
            db.insert_user(key_pair).await?;
        }

        let users = db.get_users(num_users).await?;

        Ok(Self {
            keys_path: keys_path.into(),
            api_client,
            messaging_client,
            db,
            users,
            vaults: Arc::new(RwLock::new(vaults)),
        })
    }

    pub async fn vaults(&self) -> Vec<JournalistVault> {
        let vaults = self.vaults.read().await;

        // JournalistVaults are managed as a handle to a database
        // so this isn't super inefficient and means we don't have to
        // deal with RwLock guards in the calling code
        vaults.values().cloned().collect()
    }

    pub async fn get_keys_and_profiles(
        &self,
        now: DateTime<Utc>,
    ) -> anyhow::Result<VerifiedKeysAndJournalistProfiles> {
        let keys_and_profiles = self.api_client.get_public_keys().await?;
        let anchor_org_pks = load_anchor_org_pks(&self.keys_path, now)?;

        Ok(keys_and_profiles.into_trusted(&anchor_org_pks, now))
    }

    pub async fn get_users(&self) -> &[User] {
        &self.users
    }
}
