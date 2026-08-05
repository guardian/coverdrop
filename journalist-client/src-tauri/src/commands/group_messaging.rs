use common::api::models::sentinel_id::SentinelIdentity;
use group_messaging_service::{Group, MlsMessageContent, MlsMessageContentWithId};
use journalist_vault::{GroupId, MessageId};
use snafu::{OptionExt as _, ResultExt};
use tauri::State;
use uuid::Uuid;

use crate::model::BackendToFrontendEvent;
use crate::{
    app_state::AppStateHandle,
    error::{CommandError, GroupMessagingSnafu, PublicInfoUnavailableSnafu},
};
// TODO make GroupMessagingService non-optional after all vaults have SentinelIdentities https://github.com/guardian/coverdrop-internal/issues/3885

/// Send a message to a group that the user has started or stopped typing.
#[tauri::command]
pub async fn user_typing(
    app: State<'_, AppStateHandle>,
    group_id: GroupId,
    is_typing: bool,
) -> Result<(), CommandError> {
    let Some(group_messaging_service) = app.inner().group_messaging_service().await else {
        tracing::info!("GroupMessagingService is unavailable");
        return Ok(());
    };

    let content = MlsMessageContent::IsTyping(is_typing);
    let content_with_id = MlsMessageContentWithId::new_from_content(content);

    group_messaging_service
        .send_message(&group_id, &content_with_id)
        .await
        .context(GroupMessagingSnafu {
            failed_to: "send group message",
        })?;

    app.app_handle
        .emit_group_change(group_id)
        .context(GroupMessagingSnafu {
            failed_to: "emit group message",
        })?;

    Ok(())
}

/// Send a Text message to a group.
#[tauri::command]
pub async fn send_group_message(
    app: State<'_, AppStateHandle>,
    group_id: GroupId,
    message: String,
) -> Result<(), CommandError> {
    let Some(group_messaging_service) = app.inner().group_messaging_service().await else {
        tracing::info!("GroupMessagingService is unavailable");
        return Ok(());
    };

    let content = MlsMessageContent::Text(message);
    let content_with_id = MlsMessageContentWithId::new_from_content(content);

    group_messaging_service
        .send_message(&group_id, &content_with_id)
        .await
        .context(GroupMessagingSnafu {
            failed_to: "send group message",
        })?;

    app.app_handle
        .emit_group_change(group_id)
        .context(GroupMessagingSnafu {
            failed_to: "emit group message",
        })?;

    Ok(())
}

#[tauri::command]
pub async fn create_group(
    app: State<'_, AppStateHandle>,
    group_members: Vec<SentinelIdentity>,
    display_name: String,
    description: String,
) -> Result<(), CommandError> {
    let Some(group_messaging_service) = app.inner().group_messaging_service().await else {
        tracing::info!("GroupMessagingService is unavailable");
        return Ok(());
    };

    let public_info = app.public_info().await;
    let public_info = public_info.as_ref().context(PublicInfoUnavailableSnafu)?;

    let group_id = GroupId::new(Uuid::new_v4().to_string());

    group_messaging_service
        .create_group_with_members(
            group_id.clone(),
            group_members,
            &display_name,
            &description,
            &public_info.keys,
        )
        .await
        .context(GroupMessagingSnafu {
            failed_to: "create group",
        })?;

    app.app_handle
        .emit_group_change(group_id.clone())
        .context(GroupMessagingSnafu {
            failed_to: "emit group message",
        })?;

    Ok(())
}

/// Modifies group membership and/or group info if either has changed.
#[tauri::command]
pub async fn modify_group(
    app: State<'_, AppStateHandle>,
    group_id: GroupId,
    new_group_members: Vec<SentinelIdentity>,
    new_group_name: String,
    new_description: String,
) -> Result<(), CommandError> {
    let Some(group_messaging_service) = app.inner().group_messaging_service().await else {
        tracing::info!("GroupMessagingService is unavailable");
        return Ok(());
    };

    let public_info = app.public_info().await;
    let public_info = public_info.as_ref().context(PublicInfoUnavailableSnafu)?;

    group_messaging_service
        .modify_group(
            &group_id,
            new_group_members,
            &new_group_name,
            &new_description,
            &public_info.keys,
        )
        .await
        .context(GroupMessagingSnafu {
            failed_to: "modify group",
        })?;

    app.app_handle
        .emit_group_change(group_id)
        .context(GroupMessagingSnafu {
            failed_to: "emit group message",
        })?;

    Ok(())
}

// get group info with messages and members for all groups.
#[tauri::command]
pub async fn get_group_with_messages(
    app: State<'_, AppStateHandle>,
    group_id: GroupId,
) -> Result<Option<Group>, CommandError> {
    let Some(group_messaging_service) = app.inner().group_messaging_service().await else {
        tracing::info!("GroupMessagingService is unavailable");
        return Ok(None); // TODO remove the Option wrapper when group_messaging_service is no longer optional
    };

    let group_with_messages = group_messaging_service
        .get_group_with_messages(group_id)
        .await
        .context(GroupMessagingSnafu {
            failed_to: "get group and messages",
        })?;

    Ok(Some(group_with_messages))
}

// get group info with messages and members for all groups.
#[tauri::command]
pub async fn get_groups_and_messages(
    app: State<'_, AppStateHandle>,
) -> Result<Vec<Group>, CommandError> {
    let Some(group_messaging_service) = app.inner().group_messaging_service().await else {
        tracing::info!("GroupMessagingService is unavailable");
        return Ok(vec![]);
    };

    let groups_with_messages = group_messaging_service
        .get_groups_and_messages()
        .await
        .context(GroupMessagingSnafu {
            failed_to: "get groups and messages",
        })?;

    Ok(groups_with_messages)
}

#[tauri::command]
pub async fn update_group_messages_read_status(
    app: State<'_, AppStateHandle>,
    group_id: GroupId,
    messages: Vec<MessageId>,
    read: bool, // true if marking as read, false if marking as unread
) -> Result<(), CommandError> {
    let Some(group_messaging_service) = app.inner().group_messaging_service().await else {
        tracing::info!("GroupMessagingService is unavailable");
        return Ok(());
    };

    group_messaging_service
        .update_group_messages_read_status(&group_id, messages, read)
        .await
        .context(GroupMessagingSnafu {
            failed_to: "update group messages read status",
        })?;

    app.app_handle
        .emit_group_change(group_id)
        .context(GroupMessagingSnafu {
            failed_to: "emit group message",
        })?;

    Ok(())
}
