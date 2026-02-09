use std::path::PathBuf;

use async_trait::async_trait;
use chrono::Duration;
use common::{
    protocol::keys::{
        load_covernode_provisioning_key_pairs_with_parent,
        load_journalist_provisioning_key_pairs_with_parent,
    },
    task::Task,
    time,
};
use identity_api_database::Database;

pub struct CheckFileSystemForKeysTask {
    interval: Duration,
    keys_path: PathBuf,
    database: Database,
}

impl CheckFileSystemForKeysTask {
    pub fn new(interval: Duration, keys_path: impl Into<PathBuf>, database: Database) -> Self {
        Self {
            interval,
            keys_path: keys_path.into(),
            database,
        }
    }
}

#[async_trait]
impl Task for CheckFileSystemForKeysTask {
    fn name(&self) -> &'static str {
        "check_file_system_for_keys"
    }

    async fn run(&self) -> anyhow::Result<()> {
        let now = time::now();

        // Load anchor org public keys from database
        let anchor_org_pks = self.database.select_anchor_organization_pks(now).await?;

        //
        // Journalist key pairs
        //

        tracing::info!("Checking for journalist provisioning key pairs");
        let journalist_provisioning_key_pairs = load_journalist_provisioning_key_pairs_with_parent(
            &self.keys_path,
            &anchor_org_pks,
            now,
        )?;
        tracing::info!(
            "Found {} journalist provisioning key pairs",
            journalist_provisioning_key_pairs.len()
        );

        for (journalist_provisioning_key_pair, anchor_org_pk) in &journalist_provisioning_key_pairs
        {
            tracing::info!("Adding journalist provisioning key pair to database");
            self.database
                .insert_journalist_provisioning_key_pair(
                    anchor_org_pk,
                    journalist_provisioning_key_pair,
                    now,
                )
                .await?;
        }

        //
        // CoverNode key pairs
        //

        tracing::info!("Checking for CoverNode provisioning key pairs");
        let covernode_provisioning_key_pairs = load_covernode_provisioning_key_pairs_with_parent(
            &self.keys_path,
            &anchor_org_pks,
            now,
        )?;
        tracing::info!(
            "Found {} CoverNode provisioning key pairs",
            covernode_provisioning_key_pairs.len()
        );

        for (covernode_provisioning_key_pair, anchor_org_pk) in &covernode_provisioning_key_pairs {
            tracing::info!("Adding CoverNode provisioning key pair to database");
            self.database
                .insert_covernode_provisioning_key_pair(
                    anchor_org_pk,
                    covernode_provisioning_key_pair,
                    now,
                )
                .await?;
        }

        //
        // Cleanup
        //

        tracing::info!("Performing cleanup of keys directory");
        let directory = std::fs::read_dir(&self.keys_path)?;

        for entry in directory {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
                tracing::info!("Deleting file system key: {}", path.display());
                std::fs::remove_file(path)?;
            }
        }

        Ok(())
    }

    fn interval(&self) -> Duration {
        self.interval
    }
}
