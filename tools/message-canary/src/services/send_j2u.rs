use std::{collections::HashSet, time::Duration};

use common::{crypto::keys::public_key::PublicKey, throttle::Throttle, time};
use coverdrop_service::JournalistCoverDropService;
use journalist_vault::VaultMessage;
use rand::seq::IteratorRandom as _;
use uuid::Uuid;

use crate::canary_state::CanaryState;

pub async fn send_j2u(canary_state: CanaryState, mph_j2u: u32) -> anyhow::Result<()> {
    let throttle_duration = Duration::from_secs(3600) / mph_j2u;
    let mut throttle = Throttle::new(throttle_duration);

    tracing::debug!(
        "Will send one j2u message every {:.2} seconds",
        throttle_duration.as_secs_f32()
    );

    let mut journalist_index: usize = 0;

    loop {
        let now = time::now();

        let keys_and_profiles = canary_state.get_keys_and_profiles(now).await?;

        let vaults = canary_state.vaults().await;
        let Some(vault) = vaults.get(journalist_index % vaults.len()) else {
            anyhow::bail!("No journalists in canary");
        };
        let coverdrop_service = JournalistCoverDropService::new(&canary_state.api_client, vault);
        journalist_index += 1;

        let journalist_id = vault.journalist_id().await?;

        let messages = vault.messages().await?;
        let users = messages
            .iter()
            .map(|msg| match msg {
                VaultMessage::U2J(msg) => &msg.user_pk,
                VaultMessage::J2U(msg) => &msg.user_pk,
            })
            .collect::<HashSet<_>>();

        if users.is_empty() {
            tracing::warn!(
                "No messages from users to journalist {}, cannot reply",
                journalist_id
            );

            throttle.wait().await;
            continue;
        }

        let Some(user_pk) = users.iter().choose(&mut rand::thread_rng()) else {
            anyhow::bail!("Failed to choose random user");
        };

        let j2u_message = Uuid::new_v4().to_string();

        tracing::info!(
            "sending j2u message {} to user {}",
            j2u_message,
            user_pk.public_key_hex()
        );

        coverdrop_service
            .enqueue_j2u_message(&keys_and_profiles, user_pk, j2u_message.as_str(), now)
            .await?;
        coverdrop_service
            .dequeue_and_send_j2u_message(&keys_and_profiles.keys, now)
            .await?;

        canary_state
            .db
            .insert_j2u_message(&journalist_id, user_pk, &j2u_message, now)
            .await?;

        throttle.wait().await;
    }
}
