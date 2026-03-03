use std::time::Duration;

use common::{throttle::Throttle, time};
use coverdrop_service::JournalistCoverDropService;

use crate::canary_state::CanaryState;

pub async fn receive_u2j(canary_state: CanaryState) -> anyhow::Result<()> {
    let journalists = canary_state.db.get_journalists().await?;
    if journalists.is_empty() {
        anyhow::bail!("No journalists found in database.")
    }

    let throttle_duration = Duration::from_secs(60);
    let mut throttle = Throttle::new(throttle_duration);

    tracing::debug!("journalists={:?}", journalists);

    loop {
        let now = time::now();

        // Get keys and profiles once for all vaults
        let public_info = canary_state.get_keys_and_profiles(now).await?;

        let vaults = canary_state.vaults().await;

        for vault in vaults {
            let journalist_id = vault.journalist_id().await?;

            // Create a CoverDropService for this vault
            let service = JournalistCoverDropService::new(&canary_state.api_client, &vault);

            // Pull and decrypt dead drops
            let decrypted_messages = service
                .pull_and_decrypt_dead_drops(&public_info, None::<fn(usize)>, time::now())
                .await?;

            tracing::info!(
                "Journalist {} decrypted {} messages",
                journalist_id,
                decrypted_messages.len()
            );

            // Process decrypted messages with canary-specific logic
            for decrypted_message in decrypted_messages {
                let message = decrypted_message.u2j_message.message.to_string()?;

                tracing::info!(
                    "journalist {} received message {} from user_id {:?} in dead drop {}",
                    journalist_id,
                    message,
                    decrypted_message.u2j_message.reply_key,
                    decrypted_message.dead_drop_id
                );

                let maybe_delivery_duration = canary_state
                    .db
                    .update_u2j_message_setting_received_at(
                        &journalist_id,
                        message.as_str(),
                        time::now(),
                    )
                    .await?;

                if let Some(delivery_duration) = maybe_delivery_duration {
                    // record delivery time metric
                    metrics::histogram!("U2JMessageDeliveryTime")
                        .record(delivery_duration.num_seconds() as f64);
                } else {
                    tracing::warn!(
                        "journalist {} received duplicate message {} from user_id {:?} in dead drop {}",
                        journalist_id,
                        message,
                        decrypted_message.u2j_message.reply_key,
                        decrypted_message.dead_drop_id
                    );
                    metrics::counter!("DuplicateU2JMessage").increment(1);
                }
            }
        }

        throttle.wait().await;
    }
}
