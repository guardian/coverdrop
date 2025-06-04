use client::commands::user::messages::send_user_to_journalist_cover_message;
use common::{
    protocol::keys::CoverDropPublicKeyHierarchy, u2j_appender::messaging_client::MessagingClient,
};

pub async fn send_user_to_journalist_cover_messages(
    messaging_client: &MessagingClient,
    keys: &CoverDropPublicKeyHierarchy,
    num_messages: usize,
) {
    for _ in 0..(num_messages) {
        send_user_to_journalist_cover_message(messaging_client, keys)
            .await
            .expect("Send user cover message")
    }
}
