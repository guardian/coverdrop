use chrono::Duration;
use common::{throttle::Throttle, time};
use coverdrop_service::JournalistCoverDropService;

use crate::canary_state::CanaryState;

/// Rotates journalist keys and clean up journalist vaults.
pub async fn rotate_journalist_keys(canary_state: CanaryState) -> anyhow::Result<()> {
    let throttle_duration = Duration::minutes(10);
    let mut throttle = Throttle::new(throttle_duration.to_std()?);

    loop {
        let vaults = canary_state.vaults().await;

        for vault in vaults {
            let now = time::now();

            let service = JournalistCoverDropService::new(&canary_state.api_client, &vault);
            service.check_and_rotate_keys(now).await?;

            vault.clean_up(now).await?;
        }

        throttle.wait().await;
    }
}
