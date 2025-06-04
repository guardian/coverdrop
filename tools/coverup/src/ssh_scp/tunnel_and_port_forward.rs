use std::net::Ipv4Addr;

use crate::{subprocess::create_subprocess, util::wait_for_port_active};

pub async fn command_over_ssh(
    ssh_user: &str,
    admin_machine_ip: Ipv4Addr,
    command: &str,
) -> anyhow::Result<()> {
    println!("Running command over SSH: {}", command);
    let ssh_command = format!("ssh {}@{} -t '{}'", ssh_user, admin_machine_ip, command);

    let mut child = create_subprocess("SSH command", ssh_command.as_str(), true).await?;

    let _ = child.wait().await;

    Ok(())
}

pub async fn tunnel_and_port_forward(
    ssh_user: &str,
    admin_machine_ip: Ipv4Addr,
    service_name: &str,
    service_namespace: &str,
    remote_port: u16,
    local_port: u16,
) -> anyhow::Result<()> {
    // We map the k8s service port to the local port on the admin machine and then
    // map that port to the same port on the coverup host machine. This is because
    // some other user might have a tunnel open on the same machine, meaning the
    // current user has to select another port, but cannot change the port on the
    // pod/service.
    //
    // POD: remote port (e.g. 80)
    // ADMIN MACHINE: local port (e.g. 8444)
    // DEV LAPTOP: local port (e.g. 8444)

    let port_forward_command = format!(
        "kubectl port-forward svc/{} -n {} {}:{}",
        service_name, service_namespace, local_port, remote_port,
    );

    let command = format!(
        "ssh -L {}:localhost:{} {}@{} -t {}",
        local_port, local_port, ssh_user, admin_machine_ip, port_forward_command
    );

    let mut child = create_subprocess("Tunnel and port-forward", command.as_str(), true).await?;

    let child_id = child
        .id()
        .ok_or_else(|| anyhow::anyhow!("Failed to get child process id"))?;

    println!("Tunnel has been created. Tunnel process id: {:?}", child_id);

    wait_for_port_active(local_port, true).await;

    open::that(format!("https://localhost:{}", local_port))?;

    let _ = child.wait().await;

    Ok(())
}
