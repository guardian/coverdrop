use tokio::process::Command;

use crate::{coverdrop_service::CoverDropService, data_copier_pod::DataCopierPod};
use std::path::Path;

pub async fn data_copier_shell(
    service: CoverDropService,
    kubeconfig_path: &Option<impl AsRef<Path>>,
) -> anyhow::Result<()> {
    tracing::debug!("Creating or getting data copier pod");
    let data_copier_pod =
        DataCopierPod::get_or_create(service.as_pvc_name(), kubeconfig_path).await?;
    tracing::debug!("Got data copier pod");

    tracing::debug!("Running exec");
    let pod_name = data_copier_pod.name();

    let exec_args = vec!["exec", "-itn", "on-premises", &pod_name, "--", "/bin/sh"];

    let status = Command::new("kubectl").args(&exec_args).status().await?;

    if !status.success() {
        anyhow::bail!("Failed to exec into pod");
    }
    Ok(())
}
