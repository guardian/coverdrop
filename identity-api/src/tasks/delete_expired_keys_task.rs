use async_trait::async_trait;
use chrono::Duration;
use common::{task::Task, time};
use identity_api_database::Database;

pub struct DeleteExpiredKeysTask {
    interval: Duration,
    database: Database,
}

impl DeleteExpiredKeysTask {
    pub fn new(interval: Duration, database: Database) -> Self {
        Self { interval, database }
    }
}

#[async_trait]
impl Task for DeleteExpiredKeysTask {
    fn name(&self) -> &'static str {
        "delete_expired_keys"
    }

    async fn run(&self) -> anyhow::Result<()> {
        let now = time::now();

        let deleted_journalist_provisioning_key_pairs = self
            .database
            .delete_expired_journalist_provisioning_key_pairs(now)
            .await?;
        if deleted_journalist_provisioning_key_pairs > 0 {
            tracing::info!(
                "Deleted {} expired journalist provisioning key pairs",
                deleted_journalist_provisioning_key_pairs
            );
        }

        let deleted_covernode_provisioning_key_pairs = self
            .database
            .delete_expired_covernode_provisioning_key_pairs(now)
            .await?;
        if deleted_covernode_provisioning_key_pairs > 0 {
            tracing::info!(
                "Deleted {} expired CoverNode provisioning key pairs",
                deleted_covernode_provisioning_key_pairs
            );
        }

        let deleted_org_pks = self.database.delete_expired_orgaization_pks(now).await?;
        if deleted_org_pks > 0 {
            tracing::info!(
                "Deleted {} expired organization public keys",
                deleted_org_pks
            );
        }

        Ok(())
    }

    fn interval(&self) -> Duration {
        self.interval
    }
}
