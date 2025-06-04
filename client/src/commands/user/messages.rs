use common::api::models::journalist_id::JournalistIdentity;
use common::client::mailbox::user_mailbox::UserMailbox;
use common::protocol::keys::CoverDropPublicKeyHierarchy;
use common::u2j_appender::messaging_client::MessagingClient;
use common::{protocol, FixedSizeMessageText};

pub async fn send_user_to_journalist_real_message(
    messaging_client: &MessagingClient,
    mailbox: &mut UserMailbox,
    keys: &CoverDropPublicKeyHierarchy,
    journalist_id: &JournalistIdentity,
    message: &str,
) -> anyhow::Result<()> {
    let message = FixedSizeMessageText::new(message)?;
    mailbox.add_message_to_journalist_from_user(journalist_id, &message);

    let user_pk = mailbox.user_key_pair().public_key();

    let encrypted_outer_msg =
        protocol::user::encrypt_real_message_from_user_to_journalist_via_covernode(
            keys,
            user_pk,
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
