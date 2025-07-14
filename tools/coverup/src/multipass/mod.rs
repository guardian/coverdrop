use std::{
    io::Write,
    net::Ipv4Addr,
    num::NonZeroU8,
    process::{self, Stdio},
};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::docker::ImageAndTag;

pub const NODE_PREFIX: &str = "coverdrop-node";

#[derive(Debug, Serialize, Deserialize)]
pub struct MultipassNode {
    pub ipv4: Vec<Ipv4Addr>,
    pub name: String,
    pub release: String,
    pub state: String,
}

impl MultipassNode {
    pub fn local_ip(&self) -> Option<&Ipv4Addr> {
        self.ipv4
            .iter()
            .find(|ip| ip.octets().starts_with(&[192, 168, 64]))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MultipassListing {
    pub list: Vec<MultipassNode>,
}

pub fn list_coverdrop_nodes() -> anyhow::Result<Vec<MultipassNode>> {
    let output = process::Command::new("multipass")
        .arg("ls")
        .arg("--format=json")
        .output()?;

    if output.status.success() {
        let listings = serde_json::from_slice::<MultipassListing>(&output.stdout)?;

        let coverdrop_nodes: Vec<MultipassNode> = listings
            .list
            .into_iter()
            .filter(|node| node.name.starts_with(NODE_PREFIX))
            .collect();

        Ok(coverdrop_nodes)
    } else {
        anyhow::bail!("`multipass ls` exited with non-zero exit code")
    }
}

pub fn delete_node(node: &MultipassNode) -> anyhow::Result<()> {
    let output = process::Command::new("multipass")
        .arg("delete")
        .arg("-p")
        .arg(&node.name)
        .output()?;

    if !output.status.success() {
        anyhow::bail!("`multipass delete` exited with non-zero exit code")
    }

    Ok(())
}

pub fn ensure_nodes_running(
    existing_nodes: &[MultipassNode],
    desired_node_count: NonZeroU8,
    cpus_per_node: NonZeroU8,
    ram_gb_per_node: NonZeroU8,
    storage_gb_per_node: NonZeroU8,
) -> anyhow::Result<()> {
    static CLOUD_INIT: &[u8] = include_bytes!("./multipass-cloud-init.yaml");

    for i in 0..desired_node_count.get() {
        let node_name = format!("{NODE_PREFIX}{i}");

        tracing::debug!("Ensuring node {} is launched", node_name);

        if existing_nodes.iter().any(|node| node.name == node_name) {
            tracing::debug!("A node with name {} already exists", node_name);
            continue;
        }

        let mut process = process::Command::new("multipass")
            .arg("launch")
            .arg(format!("--name={node_name}"))
            .arg(format!("--cpus={}", cpus_per_node.get()))
            .arg(format!("--memory={}G", ram_gb_per_node.get()))
            .arg(format!("--disk={}G", storage_gb_per_node.get()))
            .arg("--cloud-init=-") // from stdin
            .stdin(Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = process.stdin.take() {
            stdin.write_all(CLOUD_INIT)?;
        }

        let status = process.wait()?;
        if !status.success() {
            anyhow::bail!("Failed to create node {}", i);
        }
    }

    Ok(())
}

pub fn copy_docker_image_to_nodes(
    docker_image_and_tag: &ImageAndTag,
    nodes: &[MultipassNode],
) -> anyhow::Result<()> {
    nodes
        .par_iter()
        .map(|node| {
            let mut docker_save = process::Command::new("docker")
                .arg("save")
                .arg(docker_image_and_tag.as_str())
                .stdout(Stdio::piped())
                .spawn()?;

            if let Some(docker_stdout) = docker_save.stdout.take() {
                let ip = node
                    .local_ip()
                    .ok_or_else(|| anyhow::anyhow!("Node has no local IP"))?;

                let ssh_status = process::Command::new("ssh")
                    .arg(format!("ubuntu@{ip}"))
                    .arg("sudo k3s ctr images import -")
                    .stdin(Stdio::from(docker_stdout))
                    .status()?;

                if !ssh_status.success() {
                    anyhow::bail!("Failed to import image on node {}", node.name);
                }
                tracing::info!(
                    "Successfully copied {} to {}",
                    docker_image_and_tag.as_str(),
                    ip
                );
            }

            let status = docker_save.wait()?;

            if !status.success() {
                anyhow::bail!("Failed to save docker image");
            }

            Ok(())
        })
        .collect::<anyhow::Result<()>>()?;

    Ok(())
}
