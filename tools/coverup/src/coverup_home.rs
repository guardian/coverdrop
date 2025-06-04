use crate::external_dependencies::ExternalDependency;
use common::clap::Stage;
use directories::UserDirs;
use std::{
    ffi::OsStr,
    fs::{self, File},
    hash::{DefaultHasher, Hash, Hasher},
    io::Write,
    path::PathBuf,
};

pub struct CoverUpHome {
    coverup_dir: PathBuf,
}

impl CoverUpHome {
    /// Gets or creates a `.coverup` directory in the users home directory.
    ///
    /// Some basic things such as the multipass dev SSH keys will created there too.
    ///
    /// The home directory is found using the `directories` crate
    /// https://docs.rs/directories/5.0.1/directories/struct.UserDirs.html#method.home_dir
    pub fn new() -> anyhow::Result<Self> {
        let Some(user_dir) = UserDirs::new() else {
            anyhow::bail!("User has no home directory, cannot continue!");
        };

        let home_dir = user_dir.home_dir();

        let coverup_dir = home_dir.join(".coverup");

        if !coverup_dir.exists() {
            tracing::info!("Creating coverup home directory");
            fs::create_dir(&coverup_dir)?;
        }

        let coverup_home = Self { coverup_dir };

        // Create multipass SSH keys - we do this every time so that we don't need to worry about
        // stale keys if we ever rotate the key pair used for accessing the dev setup

        let multipass_public_key_path = coverup_home.coverup_multipass_ssh_public_key();
        let multipass_secret_key_path = coverup_home.coverup_multipass_ssh_secret_key();

        tracing::info!("Creating multipass public key in coverup home directory");
        let public_key_bytes = include_bytes!("multipass/multipass-ssh-key.pub");
        fs::write(multipass_public_key_path, public_key_bytes)?;

        tracing::info!("Creating multipass secret key in coverup home directory");
        let secret_key_bytes = include_bytes!("multipass/multipass-ssh-key");
        fs::write(multipass_secret_key_path, secret_key_bytes)?;

        Ok(coverup_home)
    }

    const DEPENDENCIES_CACHE_FILE_EXTENSION: &str = "cached-deps";

    fn dependencies_cache_file_path(&self, dependencies: &[ExternalDependency]) -> PathBuf {
        let mut hasher = DefaultHasher::new();
        dependencies.hash(&mut hasher);
        let hash = hasher.finish();

        let mut path = self.coverup_dir.join(hash.to_string());
        path.set_extension(Self::DEPENDENCIES_CACHE_FILE_EXTENSION);

        path
    }

    pub fn cached_dependencies_file_exists(&self, dependencies: &[ExternalDependency]) -> bool {
        let path = self.dependencies_cache_file_path(dependencies);
        path.exists()
    }

    pub fn create_cached_dependencies_file(
        &self,
        dependencies: &[ExternalDependency],
    ) -> anyhow::Result<()> {
        let path = self.dependencies_cache_file_path(dependencies);

        // `OsStr::new` is not const so we have to construct it here
        let deps_ext = Some(OsStr::new(Self::DEPENDENCIES_CACHE_FILE_EXTENSION));

        fs::read_dir(&self.coverup_dir)?.for_each(|entry| {
            if let Ok(entry) = entry {
                if entry.path().extension() == deps_ext {
                    // Clean up other cached dependency files
                    _ = fs::remove_file(entry.path());
                }
            };
        });

        let mut file = File::create(path)?;

        file.write_all(
            b"This file is used to cache the result of coverups external dependencies check and can be ignored\n",
        )?;

        Ok(())
    }

    pub fn kubeconfig_path_for_stage(&self, stage: Stage) -> anyhow::Result<PathBuf> {
        let path = self.kubeconfig_for_stage(stage);

        if !path.exists() {
            anyhow::bail!("Kubeconfig file {:?} does not exist. Use the command `coverup {} kubeconfig` to fetch it",  path, stage.as_clap_str());
        }

        Ok(path)
    }

    pub fn kubeconfig_path_for_optional_stage(
        &self,
        stage: Option<Stage>,
    ) -> anyhow::Result<Option<PathBuf>> {
        stage
            .map(|stage| self.kubeconfig_path_for_stage(stage))
            .transpose()
    }

    pub fn kubeconfig_for_stage(&self, stage: Stage) -> PathBuf {
        self.coverup_dir
            .join(format!("kubeconfig-{}", stage.as_guardian_str()))
    }

    // TODO: Maybe remove? Since Moving off the signal bridge we don't really need coverup to help manage vaults
    pub fn _coverup_vaults_directory(&self, stage: Stage) -> anyhow::Result<PathBuf> {
        let vaults_dir = self
            .coverup_dir
            .join(format!("{}-vaults", stage.as_guardian_str()));

        if !vaults_dir.exists() {
            tracing::info!("Creating coverup vaults directory");
            fs::create_dir(&vaults_dir)?;
        }

        Ok(vaults_dir)
    }

    pub fn _coverup_keys_directory(&self, stage: Stage) -> anyhow::Result<PathBuf> {
        let keys_dir = self
            .coverup_dir
            .join(format!("{}-keys", stage.as_guardian_str()));

        if !keys_dir.exists() {
            anyhow::bail!("Directory {} doesn't exist - it should have been created by the set-up-dev process", keys_dir.as_os_str().to_string_lossy());
        }

        Ok(keys_dir)
    }

    pub fn coverup_multipass_ssh_public_key(&self) -> PathBuf {
        self.coverup_dir.join("coverup-multipass-ssh.pub")
    }

    pub fn coverup_multipass_ssh_secret_key(&self) -> PathBuf {
        self.coverup_dir.join("coverup-multipass-ssh")
    }
}
