use crate::{
    data_copier_pod::DataCopierPod,
    local_or_pvc_path::{LocalOrPvcPath, PvcPath},
};
use std::path::Path;

pub async fn copy_file(
    source: &LocalOrPvcPath,
    destination: &LocalOrPvcPath,
    force: bool,
    kubeconfig_path: &Option<impl AsRef<Path>>,
) -> anyhow::Result<()> {
    let pvc = match (source, destination) {
        (LocalOrPvcPath::Local { .. }, LocalOrPvcPath::Pvc(PvcPath { pvc, .. })) => pvc,
        (LocalOrPvcPath::Pvc(PvcPath { pvc, .. }), LocalOrPvcPath::Local { .. }) => pvc,
        _ => anyhow::bail!("Does not support copying from PVC to PVC or from local to local"),
    };

    tracing::debug!("Creating or getting data copier pod");
    let data_copier_pod = DataCopierPod::get_or_create(pvc, kubeconfig_path).await?;
    tracing::debug!("Got data copier pod");

    // Do copy
    match (source, destination) {
        // Copying to PVC from local
        (
            LocalOrPvcPath::Local { path: local_path },
            LocalOrPvcPath::Pvc(PvcPath { path: pvc_path, .. }),
        ) => {
            if !local_path.exists() {
                anyhow::bail!("Local file {} does not exist", local_path.display());
            }
            tracing::debug!("Local file exists");

            // if we want a directory above the file in the PVC,
            // create that and set the permissions and user
            if let Some(pvc_dir) = pvc_path.parent() {
                tracing::debug!("Creating intermediate directories in PVC");
                data_copier_pod.mkdir_in_pvc(pvc_dir).await?;
            }

            // Check if the file already exists in the PVC if we're not forcing the copy
            if !force {
                tracing::debug!("Checking if file already exists in PVC");
                let file_exists_in_pvc = data_copier_pod.file_exists_in_pvc(pvc_path).await?;

                if file_exists_in_pvc {
                    anyhow::bail!("File in PVC '{}' already exists, if you wish to overwrite the file rerun the command with --force", local_path.display());
                }
            }

            tracing::debug!("Copying file");
            data_copier_pod.copy_to_pod(local_path, pvc_path).await?;

            // We want al files in the PVC to be owned by the chainguard user
            // and have its permissions set to rwX for the user only, groups
            // and others have no access permissions.

            tracing::debug!("Recursively chowning entire PVC");
            data_copier_pod.recursive_chown_in_pvc("/").await?;

            tracing::debug!("Recursively chmoding entire PVC");
            data_copier_pod.recursive_chmod_in_pvc("/").await?;
        }
        // Copying to local from PVC
        (
            LocalOrPvcPath::Pvc(PvcPath { path: pvc_path, .. }),
            LocalOrPvcPath::Local { path: local_path },
        ) => {
            if local_path.exists() && !force {
                anyhow::bail!("Local file '{}' already exists, if you wish to overwrite the file rerun the command with --force", local_path.display());
            }

            data_copier_pod.copy_from_pod(local_path, pvc_path).await?;
        }
        _ => anyhow::bail!("Does not support copying from PVC to PVC or from local to local"),
    };

    Ok(())
}
