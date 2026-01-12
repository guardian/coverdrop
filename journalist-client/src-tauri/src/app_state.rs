use common::{
    api::api_client::ApiClient,
    client::VerifiedKeysAndJournalistProfiles,
    generators::NameGenerator,
    task::{RunnerMode, TaskRunner},
    time,
};
use journalist_vault::JournalistVault;
use reqwest::Url;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
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
        AutomatedBackups, BackupManager, CleanUpVault, PullDeadDrops, RefreshPublicInfo,
        RotateJournalistKeys, SendJournalistMessages, SyncJournalistProvisioningPublicKeys,
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
        is_soft_locked: bool,
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
    pub notifications: Notifications,
    inner: RwLock<AppState>,
    pub logs: LogReceiver,
    no_background_tasks: bool,
    pub backup_manager: BackupManager,
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
            backup_manager: BackupManager::new(),
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
                let rotate_keys_task = RotateJournalistKeys::new(
                    &self.backup_manager,
                    &self.app_handle,
                    &api_client,
                    &vault,
                    &path,
                    &self.public_info,
                );
                let automated_backups_task = AutomatedBackups::new(
                    &self.backup_manager,
                    &self.app_handle,
                    &api_client,
                    &vault,
                    &path,
                    &self.public_info,
                );
                let clean_up_vault_task = CleanUpVault::new(&vault);

                async move {
                    // Pulling public info *MUST* be the first task so that the pull_dead_drops_task
                    // and rotate_keys_task have fresh keys.
                    runner.add_task(refresh_public_info_task).await;
                    runner.add_task(sync_public_keys_task).await;
                    runner.add_task(pull_dead_drops_task).await;
                    runner.add_task(send_journalist_messages_task).await;
                    // Clean and vacuum vault before rotating keys (which might perform a backup) and backup task
                    runner.add_task(clean_up_vault_task).await;
                    // The following two tasks may perform a backup which is probably the slowest of all of these operations.
                    // Do them LAST so the user can see new messages while backups are running.
                    runner.add_task(rotate_keys_task).await;
                    runner.add_task(automated_backups_task).await;

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
            is_soft_locked: false,
        };

        Ok((vault, api_client))
    }

    pub async fn vault_state(&self) -> anyhow::Result<Option<VaultState>> {
        let guard = self.inner.read().await;

        if let AppState::LoggedIn {
            vault,
            path,
            is_soft_locked,
            ..
        } = &*guard
        {
            Ok(Some(VaultState {
                id: vault.journalist_id().await?.to_string(),
                path: path.clone(),
                is_soft_locked: *is_soft_locked,
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

    pub async fn soft_lock_vault(&self) -> anyhow::Result<Option<VaultState>> {
        let mut guard = self.inner.write().await;

        match &mut *guard {
            AppState::LoggedIn { is_soft_locked, .. } => {
                *is_soft_locked = true;
            }
            AppState::LoggedOut => anyhow::bail!("Not logged in"),
        }

        drop(guard);
        self.vault_state().await
    }

    pub async fn unlock_soft_locked_vault(
        &self,
        password: &str,
    ) -> anyhow::Result<Option<VaultState>> {
        let mut guard = self.inner.write().await;

        match &mut *guard {
            AppState::LoggedIn {
                path,
                is_soft_locked,
                vault,
                ..
            } => {
                if vault.check_password(path, password).await {
                    *is_soft_locked = false;
                }
            }
            AppState::LoggedOut => anyhow::bail!("Not logged in"),
        }

        drop(guard);
        self.vault_state().await
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
