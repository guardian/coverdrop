use common::aws::kinesis::client::KinesisClient;
use common::u2j_appender::messaging_client::MessagingClient;

use crate::state::CoverTrafficState;

pub async fn send_cover_traffic_in_burst(
    messaging_client: MessagingClient,
    kinesis_client: KinesisClient,
    state: CoverTrafficState,
    num_u2j: u32,
    num_j2u: u32,
) -> anyhow::Result<()> {
    tracing::debug!("num_u2j={num_u2j}, num_j2u={num_j2u}");

    // First send all the cover traffic to the user-facing side of the CoverNode
    for _ in 0..num_u2j {
        let msg = state
            .create_user_to_journalist_cover_message()
            .await
            .expect("Create U2J cover message");
        messaging_client
            .post_user_message(msg)
            .await
            .expect("Send U2J message");
    }
    tracing::info!("Sent {num_u2j} U2J cover messages");

    // Then send all the cover traffic to the journalist-facing side of the CoverNode
    for _ in 0..num_j2u {
        let msg = state
            .create_journalist_to_user_cover_message()
            .await
            .expect("Create J2U cover message");
        kinesis_client
            .encode_and_put_journalist_message(msg)
            .await
            .expect("Send J2U message");
    }
    tracing::info!("Sent {num_j2u} J2U cover messages");

    Ok(())
}
