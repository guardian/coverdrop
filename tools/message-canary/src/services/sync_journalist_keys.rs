use std::time::Duration;

use common::{throttle::Throttle, time};

use crate::canary_state::CanaryState;

/// Finds provisioning keys which have been added to the API but which have not yet been added to the journalist vault.
/// Any new provisioning keys that can be verified by a trust anchor in the vault are inserted into the vault.
pub async fn sync_journalist_provisioning_pks(canary_state: CanaryState) -> anyhow::Result<()> {
    let throttle_duration = Duration::from_secs(10 * 60);
    let mut throttle = Throttle::new(throttle_duration);

    loop {
        let now = time::now();
        let keys_and_profiles = canary_state.get_keys_and_profiles(now).await;

        match keys_and_profiles {
            Ok(keys_and_profiles) => {
                let keys = keys_and_profiles.keys;
                let api_journalist_provisioning_pks: Vec<_> =
                    keys.journalist_provisioning_pk_iter().collect();

                let vaults = canary_state.vaults().await;

                for vault in vaults {
                    vault
                        .sync_journalist_provisioning_pks(&api_journalist_provisioning_pks, now)
                        .await?;
                }
            }
            Err(e) => {
                tracing::error!("Failed to fetch keys from API {}", e);
            }
        }
        throttle.wait().await;
    }
}
