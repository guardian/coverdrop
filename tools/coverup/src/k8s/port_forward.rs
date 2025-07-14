//! Port forwarding for locally accessible k8s services
//! See ssh_scp for a similar module that works with remote services
//! over SSH
use std::path::Path;

use crate::subprocess::{create_subprocess, wait_for_subprocess};
use tokio::process::Child;

async fn port_forward(
    target_id: &str,
    namespace: &str,
    local_port: u16,
    remote_port: u16,
    kubeconfig_path: impl AsRef<Path>,
    wait_until_sigint: bool,
) -> anyhow::Result<Child> {
    let kubeconfig_path = kubeconfig_path.as_ref().display();

    let command = format!(
        "kubectl port-forward {target_id} -n {namespace} {local_port}:{remote_port} --kubeconfig {kubeconfig_path}"
    );

    println!("Setting up port forwarding. Equivalent kubectl command: {command}");

    let child = create_subprocess("Port-forward", &command, true).await?;
    println!("\n      Port forwarding has been set up at http://localhost:{local_port}\n");
    if wait_until_sigint {
        Ok(wait_for_subprocess(child, "Port-forward").await?)
    } else {
        Ok(child)
    }
}

pub async fn port_forward_argo(local_port: u16, kubeconfig_path: &Path) -> anyhow::Result<Child> {
    port_forward(
        "svc/argocd-server",
        "argocd",
        local_port,
        443,
        kubeconfig_path,
        true,
    )
    .await
}

pub async fn port_forward_longhorn(
    local_port: u16,
    kubeconfig_path: &Path,
) -> anyhow::Result<Child> {
    port_forward(
        "svc/longhorn-frontend",
        "longhorn-system",
        local_port,
        80,
        kubeconfig_path,
        true,
    )
    .await
}

pub async fn port_forward_kubernetes_dashboard(
    local_port: u16,
    kubeconfig_path: &Path,
) -> anyhow::Result<Child> {
    let kubeconfig_path_str = kubeconfig_path
        .as_os_str()
        .to_str()
        .expect("Get path from kubeconfig_path");
    let token_command = format!(
        "kubectl -n kubernetes-dashboard create token admin-user --kubeconfig {kubeconfig_path_str}"
    );
    let mut token_process = create_subprocess("Token request", &token_command, true).await?;
    token_process.wait().await?;
    port_forward(
        "svc/kubernetes-dashboard-kong-proxy",
        "kubernetes-dashboard",
        local_port,
        443,
        kubeconfig_path,
        true,
    )
    .await
}
