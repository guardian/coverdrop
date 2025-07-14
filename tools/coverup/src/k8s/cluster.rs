use crate::commands::staging_commands::kubectl_tunnel;
use crate::subprocess::create_subprocess;
use common::clap::AwsConfig;
use std::future::Future;
use std::path::Path;
use std::time::Duration;
use tokio::process::Child;

pub async fn check_cluster_available(kubeconfig_path: &Path) -> anyhow::Result<bool> {
    println!(
        "Running kubectl get pods to check connection to cluster - you can ignore stdout below..."
    );
    let kubeconfig_path = kubeconfig_path
        .as_os_str()
        .to_str()
        .expect("Get path from kubeconfig_path");

    let get_pods_command = format!("kubectl get pods --kubeconfig {kubeconfig_path}");
    let mut child = create_subprocess("Get pods", &get_pods_command, true).await?;
    let get_pods_exit_status = child.wait().await?;
    if !get_pods_exit_status.success() {
        println!(
            "Failed to get pods from context {kubeconfig_path} - either tunnel isn't running or your context file needs replacing (see coverup 'kubeconfig' command for your stage)"
        );
        return Err(anyhow::anyhow!(
            "kubectl get pods command had error status code"
        ));
    }
    Ok(true)
}

pub async fn initialise_kubectl(
    aws_config: AwsConfig,
    kubeconfig_path: &Path,
) -> anyhow::Result<Option<Child>> {
    let cluster_available = check_cluster_available(kubeconfig_path).await;

    if cluster_available.is_err() {
        println!("Cluster unavailable. Attempting to start tunnel to cluster...");
        let child = kubectl_tunnel(aws_config, 16443).await?;
        for index in 1..6 {
            let delay = index * 3;
            tokio::time::sleep(Duration::from_secs(3)).await;
            if check_cluster_available(kubeconfig_path)
                .await
                .unwrap_or(false)
            {
                return Ok(Some(child));
            }
            println!("Cluster still unavailable, trying again in {delay} seconds...");
        }
    } else {
        return Ok(None);
    }

    anyhow::bail!(
        "Failed to connect to cluster with context file {}",
        kubeconfig_path.to_str().unwrap()
    )
}

// set up a connection to kubernetes, run an action, then when that action has finished close the tunnel
pub async fn tunnel_and_run<F, Fut>(
    aws_config: AwsConfig,
    kubeconfig_path: &Path,
    action: F,
) -> anyhow::Result<()>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = ()>,
{
    let tunnel_process = initialise_kubectl(aws_config, kubeconfig_path).await?;
    action().await;
    match tunnel_process {
        Some(mut child) => {
            child.kill().await?;
            child.wait().await?;
            Ok(())
        }
        None => {
            println!("Tunnel was started outside of this process - leaving it untouched");
            Ok(())
        }
    }
}
