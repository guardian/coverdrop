use std::time::Duration;

use common::{
    api::models::messages::journalist_to_user_message::JournalistToUserMessage,
    protocol::{
        covernode::verify_journalist_to_user_dead_drop_list,
        user::get_decrypted_user_dead_drop_message,
    },
    throttle::Throttle,
    time,
};

use crate::canary_state::CanaryState;

pub async fn receive_j2u(canary_state: CanaryState) -> anyhow::Result<()> {
    let users = canary_state.get_users().await;

    let throttle_duration = Duration::from_secs(60);
    let mut throttle = Throttle::new(throttle_duration);

    loop {
        let now = time::now();

        let keys = canary_state.get_keys_and_profiles(now).await?.keys;

        let ids_greater_than = canary_state.db.get_max_j2u_dead_drop_id().await?;

        tracing::info!("pulling dead drops with id > {}", ids_greater_than);
        let dead_drop_list = canary_state
            .api_client
            .pull_user_dead_drops(ids_greater_than)
            .await?;

        let num_dead_drops = dead_drop_list.dead_drops.len();
        tracing::info!("pulled {} new dead drops", num_dead_drops);

        if dead_drop_list.dead_drops.is_empty() {
            throttle.wait().await;
            continue;
        }

        let verified_dead_drops =
            verify_journalist_to_user_dead_drop_list(&keys, &dead_drop_list, now);

        tracing::info!("Found {} verified dead drops", verified_dead_drops.len());

        let Some(max_dead_drop_id) = verified_dead_drops
            .iter()
            .max_by_key(|d| d.id)
            .map(|d| d.id)
        else {
            tracing::info!("No verified dead drops in dead drop list");

            throttle.wait().await;
            continue;
        };

        // Not all dead drops are verified, log an error and skip
        if verified_dead_drops.len() != num_dead_drops {
            tracing::error!(
                "only {} out of {} dead drops verified, skipping processing",
                verified_dead_drops.len(),
                num_dead_drops
            );
            throttle.wait().await;
            continue;
        }

        for user in users {
            for dead_drop in &verified_dead_drops {
                for msg in &dead_drop.data.messages {
                    if let Ok(Some((journalist_id, JournalistToUserMessage::Message(message)))) =
                        get_decrypted_user_dead_drop_message(&user.key_pair, &keys, msg)
                    {
                        let message_string = message.to_string()?;

                        tracing::info!(
                            "user_id {} received message {} from journalist_id {} in dead drop {}",
                            user.user_id,
                            message_string,
                            journalist_id,
                            dead_drop.id
                        );

                        let delivery_duration = canary_state
                            .db
                            .update_j2u_message_setting_received_at(
                                user.user_id,
                                &journalist_id,
                                &message_string,
                                now,
                            )
                            .await?;

                        if let Some(delivery_duration) = delivery_duration {
                            metrics::histogram!("J2UMessageDeliveryTime")
                                .record(delivery_duration.num_seconds() as f64);
                        } else {
                            tracing::warn!(
                                "user_id {} received duplicate message {} from journalist_id {} in dead drop {}",
                                user.user_id,
                                message_string,
                                journalist_id,
                                dead_drop.id
                           );
                            metrics::counter!("DuplicateJ2UMessage").increment(1);
                        }
                    }
                }
            }
        }

        tracing::info!("updating max dead drop id to {}", max_dead_drop_id);
        canary_state
            .db
            .insert_j2u_processed_dead_drop(&max_dead_drop_id, now)
            .await?;

        throttle.wait().await;
    }
}
