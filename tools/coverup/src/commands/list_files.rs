use crate::{data_copier_pod::DataCopierPod, local_or_pvc_path::PvcPath};
use std::path::Path;

pub async fn list_files(
    pvc_path: &PvcPath,
    long: bool,
    kubeconfig_path: Option<impl AsRef<Path>>,
) -> anyhow::Result<()> {
    let data_copier_pod = DataCopierPod::get_or_create(&pvc_path.pvc, &kubeconfig_path).await?;

    let files = data_copier_pod.list_files(&pvc_path.path, long).await?;

    for f in &files {
        println!("{f}");
    }

    Ok(())
}
