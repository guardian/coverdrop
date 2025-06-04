use crate::subprocess::create_subprocess;
use std::{net::Ipv4Addr, path::Path};

#[derive(Debug, Eq, PartialEq)]
pub enum ScpDirection {
    RemoteToLocal,
    // LocalToRemote,
}

pub async fn scp(
    remote_instance_ip: &Ipv4Addr,
    user: String,
    source_path: &str,
    dest_path: &Path,
    direction: ScpDirection,
    ssh_key_path: &Path,
) -> anyhow::Result<()> {
    let dest_path = dest_path
        .as_os_str()
        .to_str()
        .expect("Convert path to string");
    let source_dest = if direction == ScpDirection::RemoteToLocal {
        format!(
            "{}@{}:{} {}",
            user, remote_instance_ip, source_path, dest_path
        )
    } else {
        format!(
            "{}, {}@{}:{}",
            source_path, user, remote_instance_ip, dest_path
        )
    };
    let key_string = format!(
        "-i {}",
        ssh_key_path
            .as_os_str()
            .to_str()
            .expect("Get key path from supplied path")
    );

    let scp_command = format!("scp {} {}", key_string, source_dest);
    let mut result = create_subprocess("scp", &scp_command, true).await?;
    let exit_code = result.wait().await?;
    if !exit_code.success() {
        anyhow::bail!(
            "SCP command exited with status {}. Does the source file exist?",
            exit_code.to_string()
        )
    }
    println!("Successfully copied file {} to {}", source_path, dest_path);
    Ok(())
}
