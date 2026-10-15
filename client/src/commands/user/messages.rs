use common::api::models::journalist_id::JournalistIdentity;
use common::client::mailbox::user_mailbox::UserMailbox;
use common::protocol::keys::CoverDropPublicKeyHierarchy;
use common::u2j_appender::messaging_client::MessagingClient;
use common::{protocol, FixedSizeAuthenticatedMessageText};

pub async fn send_user_to_journalist_real_message(
    messaging_client: &MessagingClient,
    mailbox: &mut UserMailbox,
    keys: &CoverDropPublicKeyHierarchy,
    journalist_id: &JournalistIdentity,
    message: &str,
) -> anyhow::Result<()> {
    let message = FixedSizeAuthenticatedMessageText::new(message)?;

    mailbox.add_message_to_journalist_from_user(journalist_id, &message.pad_to_plain());

    let user_key_pair = mailbox.user_key_pair();

    let encrypted_outer_msg =
        protocol::user::encrypt_real_authenticated_message_from_user_to_journalist_via_covernode(
            keys,
            user_key_pair,
            journalist_id,
            message,
        )?;

    messaging_client
        .post_user_message(encrypted_outer_msg)
        .await?;

    Ok(())
}

pub async fn send_user_to_journalist_cover_message(
    messaging_client: &MessagingClient,
    keys: &CoverDropPublicKeyHierarchy,
) -> anyhow::Result<()> {
    let msg = protocol::user::new_encrypted_cover_message_from_user_via_covernode(keys)?;

    messaging_client.post_user_message(msg).await?;

    Ok(())
}
