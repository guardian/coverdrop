use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use common::{
    api::api_client::ApiClient,
    client::VerifiedKeysAndJournalistProfiles,
    generators::NameGenerator,
    task::{RunnerMode, TaskRunner},
    time,
};
use journalist_vault::JournalistVault;
use reqwest::Url;
use tauri::AppHandle;
use tokio::{
    sync::{RwLock, RwLockReadGuard},
    task::JoinHandle,
};

use crate::{
    logging::LogReceiver,
    model::VaultState,
    notifications::Notifications,
    tasks::{
        CleanUpVault, PullDeadDrops, RefreshPublicInfo, RotateJournalistKeys,
        SendJournalistMessages, SyncJournalistProvisioningPublicKeys,
    },
};

// Controls the state of the app.
#[derive(Default)]
pub enum AppState {
    #[default]
    LoggedOut,
    /// User has unlocked a vault, selected their API, etc.
    LoggedIn {
        /// Path to vault
        path: PathBuf,
        /// Handle to background task runner, useful for aborting maybe?
        _runner_join_handle: Option<JoinHandle<()>>,
        /// The users vault - unlocked
        vault: JournalistVault,
        api_client: ApiClient,
    },
}

#[derive(Default, Clone)]
pub struct PublicInfo(Arc<RwLock<Option<VerifiedKeysAndJournalistProfiles>>>);

impl PublicInfo {
    pub async fn set(&self, public_info: VerifiedKeysAndJournalistProfiles) {
        let mut guard = self.0.write().await;
        *guard = Some(public_info);
    }

    pub async fn get(&self) -> RwLockReadGuard<'_, Option<VerifiedKeysAndJournalistProfiles>> {
        self.0.read().await
    }
}

pub struct AppStateHandle {
    pub app_handle: AppHandle,
    pub name_generator: NameGenerator,
    public_info: PublicInfo,
    notifications: Notifications,
    inner: RwLock<AppState>,
    pub logs: LogReceiver,
    no_background_tasks: bool,
}

impl AppStateHandle {
    pub fn new(
        app_handle: AppHandle,
        notifications: Notifications,
        no_background_tasks: bool,
    ) -> Self {
        Self {
            app_handle,
            name_generator: NameGenerator::default(),
            public_info: PublicInfo::default(),
            inner: RwLock::new(AppState::default()),
            notifications,
            logs: LogReceiver::default(),
            no_background_tasks,
        }
    }

    pub async fn unlock_vault(
        &self,
        api_url: &Url,
        path: impl AsRef<Path>,
        password: &str,
    ) -> anyhow::Result<(JournalistVault, ApiClient)> {
        tracing::debug!("Attempting to open vault: {}", path.as_ref().display());
        tracing::debug!("Using API URL: {}", api_url);

        let vault = JournalistVault::open(&path, password).await?;
        let path = path.as_ref().to_path_buf();

        tracing::debug!("Vault successfully opened!");

        let api_client = ApiClient::new(api_url.clone());

        tracing::debug!("Processing setup bundle");
        vault
            .process_vault_setup_bundle(&api_client, time::now())
            .await?;

        let runner_join_handle = if self.no_background_tasks {
            tracing::info!("Background tasks disabled via --no-background-tasks flag");
            None
        } else {
            tracing::debug!("Starting background tasks");
            Some(tokio::task::spawn({
                // We don't ever want this to run a web server so it should only ever
                // be RunnerMode::Timer
                let mut runner = TaskRunner::new(RunnerMode::Timer);

                let refresh_public_info_task =
                    RefreshPublicInfo::new(&api_client, &vault, &self.public_info);
                let sync_public_keys_task =
                    SyncJournalistProvisioningPublicKeys::new(&vault, &self.public_info);
                let pull_dead_drops_task = PullDeadDrops::new(
                    &self.app_handle,
                    &api_client,
                    &vault,
                    &self.public_info,
                    &self.notifications,
                );
                let send_journalist_messages_task = SendJournalistMessages::new(
                    &api_client,
                    &vault,
                    &self.public_info,
                    &self.app_handle,
                );
                let rotate_keys_task = RotateJournalistKeys::new(&api_client, &vault);
                let clean_up_vault_task = CleanUpVault::new(&vault);

                async move {
                    // Pulling public info *MUST* be the first task so that the pull dead drops
                    // and rotate keys tasks have fresh keys.
                    runner.add_task(refresh_public_info_task).await;

                    runner.add_task(sync_public_keys_task).await;
                    runner.add_task(pull_dead_drops_task).await;
                    runner.add_task(rotate_keys_task).await;
                    runner.add_task(send_journalist_messages_task).await;
                    runner.add_task(clean_up_vault_task).await;

                    runner.run().await;
                }
            }))
        };

        self.logs.use_vault(&vault, time::now()).await?;

        let mut guard = self.inner.write().await;

        *guard = AppState::LoggedIn {
            path,
            _runner_join_handle: runner_join_handle,
            vault: vault.clone(),
            api_client: api_client.clone(),
        };

        Ok((vault, api_client))
    }

    pub async fn vault_state(&self) -> anyhow::Result<Option<VaultState>> {
        let guard = self.inner.read().await;

        if let AppState::LoggedIn { vault, path, .. } = &*guard {
            Ok(Some(VaultState {
                id: vault.journalist_id().await?.to_string(),
                path: path.clone(),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn vault(&self) -> Option<JournalistVault> {
        let guard = self.inner.read().await;

        match &*guard {
            AppState::LoggedOut => None,
            AppState::LoggedIn { vault, .. } => Some(vault.clone()),
        }
    }

    pub async fn api_client(&self) -> Option<ApiClient> {
        let guard = self.inner.read().await;

        match &*guard {
            AppState::LoggedOut => None,
            AppState::LoggedIn { api_client, .. } => Some(api_client.clone()),
        }
    }

    pub async fn public_info(
        &self,
    ) -> RwLockReadGuard<'_, Option<VerifiedKeysAndJournalistProfiles>> {
        self.public_info.0.read().await
    }
}
