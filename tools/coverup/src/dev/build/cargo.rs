use std::{path::PathBuf, process::Output};

use serde::Deserialize;
use tokio::process;

// We could theoretically use cargo as a crate but I think it's
// preferable to just call cargo on the shell since that means less
// fussing around with matching the version of cargo in CoverUp to
// the version present in the shell which could be especially fiddly
// in our CI actions.

fn cargo_crate_id_to_path_buf(id: &str) -> PathBuf {
    // Example url string
    // "path+file:///Users/sam_cutler/code/pfi/coverdrop/admin#0.1.0"
    PathBuf::from(
        id.trim_start_matches("path+file://")
            .split('#')
            .next()
            .unwrap_or(""),
    )
}

#[derive(Debug, Deserialize)]
pub struct MetadataDependency {
    pub name: String,
    pub path: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
pub struct MetadataPackage {
    pub id: String,
    pub name: String,
    pub dependencies: Vec<MetadataDependency>,
}

impl MetadataPackage {
    pub fn path(&self) -> PathBuf {
        cargo_crate_id_to_path_buf(&self.id)
    }
}

#[derive(Debug, Deserialize)]
pub struct CargoMetadata {
    pub workspace_root: PathBuf,
    pub workspace_members: Vec<String>,
    pub packages: Vec<MetadataPackage>,
    // There are a lot more fields in the output of `cargo metadata` but we don't care about all of them
}

impl CargoMetadata {
    pub fn workspace_member_paths(&self) -> Vec<PathBuf> {
        self.workspace_members
            .iter()
            .map(|id| cargo_crate_id_to_path_buf(id))
            .collect()
    }
}

pub async fn cargo_metadata() -> anyhow::Result<CargoMetadata> {
    tracing::info!("Getting cargo metadata");

    // Check if we're in the root directory of the cargo workspace
    let workspace_check = process::Command::new("cargo")
        .arg("metadata")
        .arg("--no-deps")
        .output()
        .await?;

    if !workspace_check.status.success() {
        print_stderr_to_logs(&workspace_check);
        anyhow::bail!("Could not get cargo metadata, dumped stderr to log output!");
    }

    let metadata: CargoMetadata = serde_json::from_slice(&workspace_check.stdout)?;

    Ok(metadata)
}

fn print_stderr_to_logs(output: &Output) {
    let stderr_str = String::from_utf8_lossy(&output.stderr);
    for line in stderr_str.lines() {
        tracing::error!("{}", line);
    }
}
