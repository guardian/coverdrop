use common::{
    protocol::user::encrypt_real_message_from_user_to_journalist_via_covernode, throttle::Throttle,
    time, FixedSizeMessageText,
};
use rand::seq::SliceRandom;
use std::time::Duration;
use uuid::Uuid;

use crate::canary_state::CanaryState;

pub async fn send_u2j(canary_state: CanaryState, mph_u2j: u32) -> anyhow::Result<()> {
    let users = canary_state.get_users().await;

    let journalists = canary_state.db.get_journalists().await?;

    let throttle_duration = Duration::from_secs(3600) / mph_u2j;

    let mut throttle = Throttle::new(throttle_duration);
    tracing::debug!(
        "Will send one u2j message every {:.2} seconds",
        throttle_duration.as_secs_f32()
    );

    loop {
        let keys = canary_state.get_keys_and_profiles(time::now()).await?.keys;

        for user in users {
            let Some(journalist) = journalists.choose(&mut rand::thread_rng()) else {
                anyhow::bail!("No journalist to randomly select");
            };

            tracing::info!(
                "sending u2j message from user {} to journalist {}",
                user.user_id,
                journalist
            );

            let message = Uuid::new_v4().to_string();

            let u2j_msg = encrypt_real_message_from_user_to_journalist_via_covernode(
                &keys,
                user.key_pair.public_key(),
                journalist,
                FixedSizeMessageText::new(&message).unwrap(),
            )?;

            canary_state
                .messaging_client
                .post_user_message(u2j_msg)
                .await?;

            canary_state
                .db
                .insert_user_to_journalist_message(user.user_id, journalist, &message, time::now())
                .await?;

            throttle.wait().await;
        }
    }
}
