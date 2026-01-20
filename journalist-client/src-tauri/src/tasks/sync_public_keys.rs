use async_trait::async_trait;
use chrono::Duration;
use common::{task::Task, time};
use journalist_vault::JournalistVault;

use crate::app_state::PublicInfo;

/// Task which finds provisioning keys which have been added to the API
/// but which have not yet been added to the journalist vault.
/// Any new provisioning keys that can be verified by a trust anchor in the vault are inserted into the vault.
pub struct SyncJournalistProvisioningPublicKeys {
    vault: JournalistVault,
    public_info: PublicInfo,
}

impl SyncJournalistProvisioningPublicKeys {
    pub fn new(vault: &JournalistVault, public_info: &PublicInfo) -> Self {
        Self {
            vault: vault.clone(),
            public_info: public_info.clone(),
        }
    }
}

#[async_trait]
impl Task for SyncJournalistProvisioningPublicKeys {
    fn name(&self) -> &'static str {
        "sync_public_keys"
    }

    async fn run(&self) -> anyhow::Result<()> {
        let now = time::now();

        let public_info = self.public_info.get().await;

        if let Some(public_info) = public_info.as_ref() {
            let api_journalist_provisioning_pks: Vec<_> =
                public_info.keys.journalist_provisioning_pk_iter().collect();

            self.vault
                .sync_journalist_provisioning_pks(&api_journalist_provisioning_pks, now)
                .await?;
        }

        Ok(())
    }

    fn interval(&self) -> Duration {
        Duration::minutes(10)
    }
}
