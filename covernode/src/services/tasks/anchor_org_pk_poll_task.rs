use std::path::PathBuf;

use async_trait::async_trait;
use chrono::Duration;
use common::{
    aws::ssm::{client::SsmClient, prefix::ParameterPrefix},
    protocol::keys::{load_anchor_org_pks, load_anchor_org_pks_from_ssm},
    task::Task,
    time,
};

use crate::key_state::KeyState;

pub struct TrustedOrganizationPublicKeyPollTask {
    interval: Duration,
    keys_path: PathBuf,
    parameter_prefix: Option<ParameterPrefix>,
    key_state: KeyState,
}

impl TrustedOrganizationPublicKeyPollTask {
    pub fn new(
        interval: Duration,
        keys_path: PathBuf,
        parameter_prefix: Option<ParameterPrefix>,
        key_state: KeyState,
    ) -> Self {
        Self {
            interval,
            keys_path,
            parameter_prefix,
            key_state,
        }
    }
}

#[async_trait]
impl Task for TrustedOrganizationPublicKeyPollTask {
    fn name(&self) -> &'static str {
        "anchor_org_pk_poll"
    }

    async fn run(&self) -> anyhow::Result<()> {
        let now = time::now();
        let anchor_org_pks = if let Some(parameter_prefix) = &self.parameter_prefix {
            tracing::info!("Fetching trusted org pk from SSM");
            let ssm_client = SsmClient::new_in_aws().await;
            load_anchor_org_pks_from_ssm(&ssm_client, parameter_prefix, now).await?
        } else {
            tracing::info!("Fetching trusted org pk from disk");
            load_anchor_org_pks(&self.keys_path, now)?
        };

        let mut key_state = self.key_state.write().await;
        key_state.set_anchor_org_pks(anchor_org_pks);

        Ok(())
    }

    fn interval(&self) -> Duration {
        self.interval
    }
}
