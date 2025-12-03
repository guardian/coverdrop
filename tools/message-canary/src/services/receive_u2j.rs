use std::time::Duration;

use common::{
    api::models::messages::user_to_journalist_message_with_dead_drop_id::UserToJournalistMessageWithDeadDropId,
    protocol::{
        covernode::verify_user_to_journalist_dead_drop_list,
        journalist::get_decrypted_journalist_dead_drop_message,
    },
    throttle::Throttle,
    time,
};

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

        let keys = canary_state.get_keys_and_profiles(now).await?.keys;

        let ids_greater_than = canary_state.db.get_max_u2j_dead_drop_id().await?;

        tracing::info!(
            "Pulling dead drops with ID greater than {}",
            ids_greater_than
        );

        let dead_drop_list = canary_state
            .api_client
            .pull_journalist_dead_drops(ids_greater_than, None)
            .await?;

        let num_dead_drops = dead_drop_list.dead_drops.len();
        tracing::info!("pulled {} new dead drops", num_dead_drops);

        let verified_dead_drops =
            verify_user_to_journalist_dead_drop_list(&keys, dead_drop_list, time::now());

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

        let covernode_msg_pks = keys
            .covernode_msg_pk_iter()
            .map(|(_, msg_pk)| msg_pk)
            .collect::<Vec<_>>();

        let vaults = canary_state.vaults().await;

        for vault in vaults {
            let journalist_id = vault.journalist_id().await?;

            let journalist_msg_key_pairs = vault
                .msg_key_pairs_for_decryption(time::now())
                .await?
                .collect::<Vec<_>>();

            let decrypted_messages: Vec<UserToJournalistMessageWithDeadDropId> =
                verified_dead_drops
                    .iter()
                    .flat_map(|dead_drop| {
                        dead_drop
                            .data
                            .messages
                            .iter()
                            .filter_map(|encrypted_message| {
                                get_decrypted_journalist_dead_drop_message(
                                    &covernode_msg_pks,
                                    &journalist_msg_key_pairs,
                                    encrypted_message,
                                    dead_drop.id,
                                )
                            })
                    })
                    .collect();

            tracing::info!(
                "Journalist {} decrypted {} messages",
                journalist_id,
                decrypted_messages.len()
            );

            vault
                .add_messages_from_user_to_journalist_and_update_max_dead_drop_id(
                    &decrypted_messages,
                    max_dead_drop_id,
                    now,
                )
                .await?;

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

        tracing::info!("updating max dead drop id to {}", max_dead_drop_id);
        canary_state
            .db
            .insert_u2j_processed_dead_drop(&max_dead_drop_id, now)
            .await?;

        throttle.wait().await;
    }
}
