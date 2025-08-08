use chrono::{DateTime, Utc};
use common::{
    client::mailbox::mailbox_message::UserStatus as MailboxUserStatus,
    protocol::{
        constants::MESSAGE_PADDING_LEN,
        journalist::{
            encrypt_real_message_from_journalist_to_user_via_covernode,
            new_encrypted_cover_message_from_journalist_via_covernode,
        },
        keys::UserPublicKey,
    },
    time, Error as CommonError, FixedSizeMessageText,
};
use journalist_vault::VaultMessage;
use snafu::{OptionExt as _, ResultExt};
use tauri::State;

use crate::{
    app_state::AppStateHandle,
    error::{
        AnyhowSnafu, ApiClientUnavailableSnafu, CommandError, CommonSnafu, GenericSnafu,
        PublicInfoUnavailableSnafu, VaultLockedSnafu, VaultSnafu,
    },
    model::{User, UserStatus},
};

#[tauri::command]
pub fn check_message_length(message: String) -> Result<f32, CommandError> {
    let padded = FixedSizeMessageText::new(&message);

    match padded {
        Ok(compressed) => {
            let compressed_len = compressed.compressed_data_len().context(CommonSnafu)?;

            let padding_len = MESSAGE_PADDING_LEN as f32;
            let ratio = compressed_len as f32 / padding_len;

            Ok(ratio)
        }
        Err(CommonError::CompressedStringTooLong(ratio)) => Ok(ratio),
        Err(e) => Err(e).context(CommonSnafu)?,
    }
}

#[tauri::command]
pub async fn get_chats(app: State<'_, AppStateHandle>) -> Result<Vec<VaultMessage>, CommandError> {
    let vault = app.inner().vault().await.context(VaultLockedSnafu)?;

    vault.messages().await.context(VaultSnafu {
        failed_to: "get messages",
    })
}

#[tauri::command]
pub async fn get_users(app: State<'_, AppStateHandle>) -> Result<Vec<User>, CommandError> {
    let vault = app.inner().vault().await.context(VaultLockedSnafu)?;

    let vault_users = vault.users().await.context(VaultSnafu {
        failed_to: "get users",
    })?;

    let users = vault_users
        .into_iter()
        .map(|user| {
            User::new(
                &app.name_generator,
                user.user_pk,
                UserStatus::from_mailbox_message_user_status(user.status),
                user.alias,
                user.description,
            )
        })
        .collect();

    Ok(users)
}

#[tauri::command]
pub async fn mark_as_read(
    app: State<'_, AppStateHandle>,
    message_id: i64,
) -> Result<(), CommandError> {
    let vault = app.inner().vault().await.context(VaultLockedSnafu)?;

    tracing::info!("Marking message {} as read", message_id);

    vault.mark_as_read(message_id).await.context(VaultSnafu {
        failed_to: "mark message as read",
    })?;

    Ok(())
}

#[tauri::command]
pub async fn mark_as_unread(
    app: State<'_, AppStateHandle>,
    message_id: i64,
) -> Result<(), CommandError> {
    let vault = app.inner().vault().await.context(VaultLockedSnafu)?;

    tracing::info!("Marking message {} as unread", message_id);

    vault.mark_as_unread(message_id).await.context(VaultSnafu {
        failed_to: "mark message as unread",
    })?;

    Ok(())
}

#[tauri::command]
pub async fn set_custom_expiry(
    app: State<'_, AppStateHandle>,
    message: VaultMessage,
    custom_expiry: Option<DateTime<Utc>>,
) -> Result<(), CommandError> {
    let vault = app.inner().vault().await.context(VaultLockedSnafu)?;

    vault
        .set_custom_expiry(&message, custom_expiry)
        .await
        .context(VaultSnafu {
            failed_to: "set a custom expiry",
        })?;

    Ok(())
}

fn user_pk_from_hex(reply_key: &str) -> Result<UserPublicKey, CommandError> {
    let user_pk = hex::decode(reply_key).ok().context(GenericSnafu {
        ctx: "Failed to decode reply key hex",
    })?;

    let user_pk = UserPublicKey::from_bytes(&user_pk)
        .ok()
        .context(GenericSnafu {
            ctx: "User reply key is valid hex but isn't a valid public key",
        })?;

    Ok(user_pk)
}

#[tauri::command]
pub async fn update_user_status(
    app: State<'_, AppStateHandle>,
    reply_key: String,
    status: MailboxUserStatus,
) -> Result<(), CommandError> {
    let vault = app.inner().vault().await.context(VaultLockedSnafu)?;

    let user_pk = user_pk_from_hex(&reply_key)?;
    tracing::info!("Setting user {:?} status to {}", user_pk, status);

    vault
        .update_user_status(&user_pk, status)
        .await
        .context(VaultSnafu {
            failed_to: "mark message as read",
        })?;

    Ok(())
}

#[tauri::command]
pub async fn update_user_alias_and_description(
    app: State<'_, AppStateHandle>,
    reply_key: String,
    alias: String,
    description: String,
) -> Result<(), CommandError> {
    let vault = app.inner().vault().await.context(VaultLockedSnafu)?;

    let user_pk = user_pk_from_hex(&reply_key)?;

    vault
        .update_user_alias_and_description(&user_pk, &alias, &description)
        .await
        .context(VaultSnafu {
            failed_to: "update user",
        })?;

    Ok(())
}

#[tauri::command]
pub async fn submit_message(
    app: State<'_, AppStateHandle>,
    reply_key: String,
    message: String,
) -> Result<(), CommandError> {
    let vault = app.inner().vault().await.context(VaultLockedSnafu)?;

    let public_info = app.public_info().await;
    let public_info = public_info.as_ref().context(PublicInfoUnavailableSnafu)?;

    let keys = &public_info.keys;

    let now = time::now();

    let unencrypted_message = FixedSizeMessageText::new(&message)
        .ok()
        .context(GenericSnafu {
            ctx: "Failed to create fixed size message",
        })?;

    let latest_journalist_msg_key_pair = vault
        .latest_msg_key_pair(now)
        .await
        // Deal with failure to read the vault
        .context(VaultSnafu {
            failed_to: "get latest messaging key pair",
        })?
        // Deal with vault read ok but there are no keys
        .context(GenericSnafu {
            ctx: "No messaging keys in vault",
        })?;

    let user_pk = user_pk_from_hex(&reply_key)?;

    let encrypted_message = encrypt_real_message_from_journalist_to_user_via_covernode(
        keys,
        &user_pk,
        &latest_journalist_msg_key_pair,
        &unencrypted_message,
    )
    .ok()
    .context(GenericSnafu {
        ctx: "Failed to encrypt message",
    })?;

    vault
        .add_message_from_journalist_to_user_and_enqueue(
            &user_pk,
            &unencrypted_message,
            encrypted_message,
            now,
        )
        .await
        .context(VaultSnafu {
            failed_to: "enqueue message",
        })?;

    Ok(())
}

#[tauri::command]
pub async fn burst_cover_messages(
    app: State<'_, AppStateHandle>,
    count: usize,
) -> Result<(), CommandError> {
    let vault = app.inner().vault().await.context(VaultLockedSnafu)?;

    let api_client = app
        .inner()
        .api_client()
        .await
        .context(ApiClientUnavailableSnafu)?;

    let public_info = app.public_info().await;
    let public_info = public_info.as_ref().context(PublicInfoUnavailableSnafu)?;

    let keys = &public_info.keys;

    let now = time::now();
    let latest_id_key_pair = vault
        .latest_id_key_pair(now)
        .await
        // Deal with failure to read the vault
        .context(VaultSnafu {
            failed_to: "get latest identity key pair",
        })?
        // Deal with vault read ok but there are no keys
        .context(GenericSnafu {
            ctx: "No identity keys in vault, you will need to reach out to an administrator",
        })?;

    for _ in 0..count {
        let j2c_msg = new_encrypted_cover_message_from_journalist_via_covernode(keys).context(
            AnyhowSnafu {
                failed_to: "encrypt message",
            },
        )?;

        api_client
            .post_journalist_msg(j2c_msg, &latest_id_key_pair, now)
            .await
            .context(AnyhowSnafu {
                failed_to: "send cover message to API",
            })?;
    }

    Ok(())
}
