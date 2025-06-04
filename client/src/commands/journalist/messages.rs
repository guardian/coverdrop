use chrono::{DateTime, Utc};
use common::aws::kinesis::client::KinesisClient;
use common::{
    protocol::{
        self,
        journalist::encrypt_real_message_from_journalist_to_user_via_covernode,
        keys::{CoverDropPublicKeyHierarchy, UserPublicKey},
    },
    FixedSizeMessageText,
};
use journalist_vault::JournalistVault;

pub async fn send_journalist_to_user_real_message(
    kinesis_client: &KinesisClient,
    keys: &CoverDropPublicKeyHierarchy,
    vault: &JournalistVault,
    user_pk: &UserPublicKey,
    message: &str,
    now: DateTime<Utc>,
) -> anyhow::Result<()> {
    // Save to mailbox
    let message = FixedSizeMessageText::new(message)?;

    let Some(journalist_msg_key_pair) = vault.latest_msg_key_pair(now).await? else {
        anyhow::bail!("No messaging key in journalist vault");
    };

    let msg = encrypt_real_message_from_journalist_to_user_via_covernode(
        keys,
        user_pk,
        &journalist_msg_key_pair,
        &message,
    )?;

    kinesis_client
        .encode_and_put_journalist_message(msg.clone())
        .await?;

    vault
        .add_message_from_journalist_to_user_and_enqueue(user_pk, &message, msg, now)
        .await?;

    Ok(())
}

pub async fn send_journalist_to_user_cover_message(
    kinesis_client: &KinesisClient,
    keys: &CoverDropPublicKeyHierarchy,
) -> anyhow::Result<()> {
    let msg =
        protocol::journalist::new_encrypted_cover_message_from_journalist_via_covernode(keys)?;

    kinesis_client
        .encode_and_put_journalist_message(msg)
        .await?;

    Ok(())
}
