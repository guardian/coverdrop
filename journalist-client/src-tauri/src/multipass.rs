#![cfg(debug_assertions)]

//! This file is largely copied from the coverup project but with many parts removed.
//! I'd rather not include coverup as a dependency since this is only for dev and I don't want
//! to increase build times for the journalist-client any more than we have to

use std::{net::Ipv4Addr, process};

use serde::{Deserialize, Serialize};

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
            .find(|ip| ip.octets().starts_with(&[192, 168]))
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
