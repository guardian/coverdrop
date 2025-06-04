use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    time::Duration,
};

use async_tar::EntryType;
use futures::StreamExt;
use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::{AttachParams, AttachedProcess, PostParams},
    runtime::{conditions::is_pod_running, wait::await_condition},
};
use serde_json::json;
use tokio::{fs::File, task::JoinHandle, time::timeout};
use tokio_util::{
    compat::{FuturesAsyncReadCompatExt, TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt},
    io::ReaderStream,
};
use walkdir::WalkDir;

use crate::{kube_client::KubeClient, listed_file::ListedFile};

/// Our production images are based off of chainguard images which do not come with a full shell.
/// Since we often want to do things like change file permissions, it is easier to simply start a
/// temporary new pod with a more complete set of tools.
pub struct DataCopierPod {
    client: KubeClient,
    name: String,
}

impl DataCopierPod {
    const MOUNT_PATH: &str = "/data";

    /// Get or create a new pod in a background task
    pub async fn get_or_create_in_background(
        pvc: &'static str,
        kubeconfig_path: Option<PathBuf>,
    ) -> JoinHandle<anyhow::Result<DataCopierPod>> {
        tokio::task::spawn(async move { Self::get_or_create(pvc, &kubeconfig_path).await })
    }

    pub async fn get_or_create(
        pvc: &str,
        kubeconfig_path: &Option<impl AsRef<Path>>,
    ) -> anyhow::Result<DataCopierPod> {
        let client = KubeClient::new(kubeconfig_path).await?;

        let name = format!("data-copier-{pvc}");

        if client.on_premises.pods.get_opt(&name).await?.is_some() {
            Ok(Self { client, name })
        } else {
            tracing::debug!("Data copier pod for '{}' does not exist, creating", pvc);

            let pod: Pod = serde_json::from_value(json!({
                "apiVersion": "v1",
                "kind": "Pod",
                "metadata": {
                    "name": name,
                    "namespace": "on-premises",
                    "labels": {
                        "app": "data-copier"
                    }
                },
                "spec": {
                    "runtimeClassName": "data-copier",
                    "containers": [{
                      "name": name,
                      "image": "ghcr.io/guardian/coverdrop_data-copier@sha256:cea0ae6981d4d567c561baa6dec2ecce8eaddb882a04ac72ac4cbcec5b1e9786",
                      "command": ["sleep"],
                      "args": ["60m"],
                      "imagePullPolicy": "IfNotPresent",
                      "volumeMounts": [{
                          "name": "vol",
                          "mountPath": Self::MOUNT_PATH
                      }]
                    }],
                    "imagePullSecrets": [{
                        "name": "ghcr"
                    }],
                    "volumes": [{
                        "name": "vol",
                        "persistentVolumeClaim": {
                            "claimName": pvc
                        }
                    }]
                }
            }))?;

            let pp = PostParams::default();

            _ = client.on_premises.pods.create(&pp, &pod).await?;

            _ = timeout(
                Duration::from_secs(120),
                await_condition(client.on_premises.pods.clone(), &name, is_pod_running()),
            )
            .await?;

            Ok(Self { client, name })
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    // this serves as the basis for the fire-and-forget functions in this type
    async fn exec(&self, cmd: Vec<&str>) -> anyhow::Result<AttachedProcess> {
        let ap = AttachParams::default();
        let attached_process = self
            .client
            .on_premises
            .pods
            .exec(self.name(), cmd, &ap)
            .await?;

        Ok(attached_process)
    }

    async fn exec_capture_stdout(&self, cmd: Vec<&str>) -> anyhow::Result<String> {
        let mut attached_process = self.exec(cmd).await?;

        let Some(stdout) = attached_process.stdout() else {
            anyhow::bail!("Attached process did not have stdout ",)
        };

        let stdout = ReaderStream::new(stdout);

        let out = stdout
            .filter_map(|r| async { r.ok().and_then(|v| String::from_utf8(v.to_vec()).ok()) })
            .collect::<Vec<_>>()
            .await
            .join("");

        attached_process.join().await?;

        Ok(out)
    }

    pub async fn mkdir_in_pvc(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let mounted_path = format!("{}/{}", Self::MOUNT_PATH, path.as_ref().display());

        self.exec(vec!["mkdir", "-p", &mounted_path]).await?;

        Ok(())
    }

    pub async fn file_exists_in_pvc(&self, path: impl AsRef<Path>) -> anyhow::Result<bool> {
        let mounted_path = format!("{}/{}", Self::MOUNT_PATH, path.as_ref().display());

        let sh_cmd = format!(
            "if [ -f {} ]; then echo true; else echo false; fi",
            &mounted_path
        );

        let cmd = vec!["sh", "-c", &sh_cmd];

        let true_or_false = self.exec_capture_stdout(cmd).await?;

        tracing::debug!("Got '{}' from existence check", true_or_false);

        let true_or_false = true_or_false.trim();

        if true_or_false != "true" && true_or_false != "false" {
            anyhow::bail!(
                "Got unexpected output from existence command, should be true or false, got '{}' ",
                true_or_false
            )
        }

        Ok(true_or_false == "true")
    }

    pub async fn list_files(
        &self,
        path: impl AsRef<Path>,
        long: bool,
    ) -> anyhow::Result<Vec<ListedFile>> {
        let mounted_path = format!("{}/{}", Self::MOUNT_PATH, path.as_ref().display());

        if long {
            let cmd = vec!["ls", "-l", &mounted_path];

            let files = self.exec_capture_stdout(cmd).await?;

            tracing::debug!("From {} got files {}", self.name, files);

            let files = files
                .lines()
                .filter(|line| !line.is_empty())
                .flat_map(|line| {
                    if line.starts_with("total ") {
                        return None;
                    }

                    let Ok(ls_file) = ListedFile::from_ls_long_line(line) else {
                        tracing::error!("Failed to parse listed file from line: {}", line);
                        return None;
                    };

                    Some(ls_file)
                })
                .collect();

            Ok(files)
        } else {
            let cmd = vec!["ls", &mounted_path];

            let files = self.exec_capture_stdout(cmd).await?;

            tracing::debug!("From {} got files {}", self.name, files);

            Ok(files
                .lines()
                .flat_map(|line| {
                    ListedFile::from_ls_line(line).inspect_err(|_| {
                        tracing::error!("Failed to parse listed file from line: {}", line)
                    })
                })
                .collect())
        }
    }

    pub async fn recursive_chown_in_pvc(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let mounted_path = format!("{}/{}", Self::MOUNT_PATH, path.as_ref().display());

        self.exec(vec!["chown", "-R", "65532:65532", &mounted_path])
            .await?;

        Ok(())
    }

    /// Recursively change the permissions on all files from a given path.
    ///
    /// The permissions are "u=rwX,go=", meaning the user can read and write files, and execute
    /// only directories. Groups and others cannot do anything. This is to prevent any users
    /// other than the specific one which is running our application from being able to do anything
    /// with the files.
    pub async fn recursive_chmod_in_pvc(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let mounted_path = format!("{}/{}", Self::MOUNT_PATH, path.as_ref().display());

        self.exec(vec!["chmod", "-R", "u=rwX,go=", &mounted_path])
            .await?;

        Ok(())
    }

    /// Use the `async-tar` rust library to package up a local file and send it to the pod
    /// using the `exec` command. The pod then uses the CLI `tar -x` to untar the
    /// file into the mounted data directory
    pub async fn copy_to_pod(
        &self,
        local_path: impl AsRef<Path>,
        pvc_path: impl AsRef<Path>,
    ) -> anyhow::Result<()> {
        let local_path = local_path.as_ref();
        let pvc_path = pvc_path.as_ref();

        let pvc_dir = match pvc_path.parent() {
            Some(dir) => format!("{}/{}", Self::MOUNT_PATH, dir.display()),
            None => format!("{}/", Self::MOUNT_PATH),
        };

        // This function works by wrapping the local file in a tar
        // archive and streaming that data to the pod with exec over stdin
        // This is how `kubectl cp` is implemented.

        let mut attached_process = self
            .client
            .on_premises
            .pods
            .exec(
                &self.name,
                vec!["tar", "xf", "-", "-C", &pvc_dir],
                &AttachParams::default().stdin(true).stderr(false),
            )
            .await?;

        let Some(attached_process_stdin) = attached_process.stdin() else {
            anyhow::bail!("Could not get stdin for data-copier attached process")
        };

        // Stream our tar file into the `tar xf` process running inside the pod
        let mut archive = async_tar::Builder::new(attached_process_stdin.compat_write());

        // The base file name is the filename if the tranferred file is a single
        // file or the name of the directory if it is a a directory
        let Some(base_file_name) = pvc_path.file_name().or_else(|| local_path.file_name()) else {
            anyhow::bail!("Neither PVC file nor local file have a file name");
        };

        if local_path.is_dir() {
            // Walk directory, logging files that we can't read
            let walker = WalkDir::new(local_path).into_iter().filter_map(|entry| {
                entry
                    .inspect_err(|e| tracing::error!("Failed to walk file: {}", e))
                    .ok()
            });

            for entry in walker {
                if !entry.file_type().is_file() {
                    continue;
                }

                let size = entry.metadata()?.len();

                let entry_path = entry.path();
                let entry_file = File::open(entry_path).await?;

                let entry_file_metadata = entry_file.metadata().await?;

                let rel_entry_path = entry_path.strip_prefix(local_path)?;

                let tar_path = Path::new(base_file_name).join(rel_entry_path);

                let mut header = async_tar::Header::new_gnu();
                header.set_metadata(&entry_file_metadata);
                header.set_path(tar_path)?;
                header.set_size(size);
                header.set_cksum();

                archive.append(&header, entry_file.compat()).await?;
            }
        } else if local_path.is_file() {
            let size = local_path.metadata()?.len();
            let local_file = File::open(local_path).await?;

            let local_file_metadata = local_file.metadata().await?;

            let mut header = async_tar::Header::new_gnu();
            header.set_metadata(&local_file_metadata);
            header.set_path(base_file_name)?;
            header.set_size(size);
            header.set_cksum();

            archive.append(&header, local_file.compat()).await?;
        } else {
            anyhow::bail!("Local path is not a file or directory");
        }

        archive.finish().await?;

        Ok(())
    }

    /// Similar to copy_to_pod but in reverse
    pub async fn copy_from_pod(
        &self,
        local_path: impl AsRef<Path>,
        pvc_path: impl AsRef<Path>,
    ) -> anyhow::Result<()> {
        let local_path = local_path.as_ref();
        let pvc_path = pvc_path.as_ref();
        let pvc_path = match pvc_path.strip_prefix("/") {
            Ok(p) => p,
            Err(_) => pvc_path,
        };

        // We have to special case if the user wants to copy the entire root volume
        // because Path::new("/data").join("/") is equal to Path::new("/")
        let (file_to_tar, parent_dir) = if pvc_path == Path::new("/") {
            (".".to_string(), "/data".to_string())
        } else {
            let absolute_pvc_path = Path::new("/data").join(pvc_path);

            let pvc_parent = absolute_pvc_path
                .parent()
                .ok_or(anyhow::anyhow!(
                    "Could not get parent directory of: {}",
                    absolute_pvc_path.display()
                ))?
                .to_str()
                .ok_or(anyhow::anyhow!("PVC parent was not valid UTF-8"))?;

            let file_to_tar = absolute_pvc_path
                .file_name()
                .ok_or(anyhow::anyhow!("Could not get file name"))?
                .to_str()
                .ok_or(anyhow::anyhow!("File name was not valid UTF-8"))?;

            (file_to_tar.to_string(), pvc_parent.to_string())
        };

        tracing::debug!("data-copier tarring: {} from {}", file_to_tar, parent_dir);

        // Create tar command in pod
        let mut attached_process = self
            .client
            .on_premises
            .pods
            .exec(
                &self.name,
                vec!["tar", "cf", "-", &file_to_tar, "-C", &parent_dir],
                &AttachParams::default().stdout(true).stderr(false),
            )
            .await?;

        let Some(attached_process_stdout) = attached_process.stdout() else {
            anyhow::bail!("Could not get stdout for data-copier attached process")
        };

        let archive = async_tar::Archive::new(attached_process_stdout.compat());

        let mut entries = archive.entries()?;

        let mut is_first_entry = true;
        while let Some(entry) = entries.next().await {
            tracing::debug!("Found entry in tar file");

            let Ok(entry) = entry else {
                tracing::error!("Failed to read tar entry");
                continue;
            };

            let Ok(entry_path) = entry.path() else {
                tracing::error!("Failed to read path in entry");
                continue;
            };

            let entry_type = entry.header().entry_type();

            tracing::debug!(
                "Tar archive found '{:?}' at path: {}",
                entry_type,
                entry_path.display()
            );

            let normalized_local_path = if entry_path.as_os_str() == OsStr::new("./")
                // Special behaviour if it's a single file to allow the user to rename
                // the file locally.
                || (entry_type == EntryType::file() && is_first_entry)
            {
                PathBuf::from(local_path)
            } else {
                Path::new(&local_path).join(entry_path.as_ref())
            };

            if entry_type == EntryType::dir() {
                tracing::debug!("Checking if {} exists", normalized_local_path.display());
                if !normalized_local_path.exists() {
                    tracing::debug!("Creating all dirs: {}", normalized_local_path.display());
                    std::fs::create_dir_all(&normalized_local_path)?;
                }
            } else if entry_type == EntryType::file() {
                tracing::debug!("Creating file: {}", normalized_local_path.display());
                let mut local_file = File::create(normalized_local_path).await?;

                tokio::io::copy(&mut entry.compat(), &mut local_file).await?;
            } else {
                tracing::error!("Entry in tar file was not a file or directory, ignoring...");
            }

            is_first_entry = false;
        }

        Ok(())
    }
}
