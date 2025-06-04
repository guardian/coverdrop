use async_trait::async_trait;
use chrono::Duration;
use common::{
    aws::ssm::client::SsmClient,
    protocol::keys::{load_anchor_org_pks, load_anchor_org_pks_from_ssm},
    task::Task,
    time,
};

use crate::{
    anchor_org_pk_cache::AnchorOrganizationPublicKeyCache, cli::KeyLocation,
    services::database::Database,
};

/// Poll `key_location` for *new* anchor organization keys to put into the database
pub struct AnchorOrganizatioPublicKeyPollTask {
    interval: Duration,
    key_location: KeyLocation,
    anchor_org_pks: AnchorOrganizationPublicKeyCache,
    db: Database,
}

impl AnchorOrganizatioPublicKeyPollTask {
    pub fn new(
        interval: Duration,
        key_location: KeyLocation,
        anchor_org_pks: AnchorOrganizationPublicKeyCache,
        db: Database,
    ) -> Self {
        Self {
            interval,
            key_location,
            anchor_org_pks,
            db,
        }
    }
}

#[async_trait]
impl Task for AnchorOrganizatioPublicKeyPollTask {
    fn name(&self) -> &'static str {
        "anchor_org_pk_poll"
    }

    async fn run(&self) -> anyhow::Result<()> {
        let anchor_org_pks = match (
            &self.key_location.keys_path,
            &self.key_location.parameter_prefix,
        ) {
            (None, Some(prefix)) => {
                tracing::info!("Fetching trusted org pk from SSM");
                let ssm_client = SsmClient::new_in_aws().await;
                load_anchor_org_pks_from_ssm(&ssm_client, prefix, time::now()).await?
            }
            (Some(keys_path), None) => {
                tracing::info!("Fetching trusted org pk from disk: {:?}", keys_path);
                load_anchor_org_pks(keys_path, time::now())?
            }
            _ => {
                unreachable!(
                    "Either parameter_prefix or keys_path must be provided for the key location"
                )
            }
        };

        for anchor_org_pk in anchor_org_pks.iter() {
            let did_insert_org_pk = self
                .db
                .organization_key_queries
                .insert_org_pk(anchor_org_pk, time::now())
                .await?;

            if did_insert_org_pk {
                metrics::counter!("OrgPksAdded").increment(1);
            }
        }

        self.anchor_org_pks.set(anchor_org_pks).await;

        Ok(())
    }

    fn interval(&self) -> Duration {
        self.interval
    }
}
