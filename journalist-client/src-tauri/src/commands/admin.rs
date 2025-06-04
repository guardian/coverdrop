use common::{
    crypto::{human_readable_digest, keys::public_key::PublicKey as _},
    time,
};
use snafu::{OptionExt as _, ResultExt as _};
use tauri::State;

use crate::{
    app_state::AppStateHandle,
    error::{
        ApiClientUnavailableSnafu, CommandError, JsonSerializeSnafu, PublicInfoUnavailableSnafu,
        VaultLockedSnafu, VaultSnafu,
    },
    model::{SentinelLogEntry, TrustedOrganizationPublicKeyAndDigest},
};

#[tauri::command]
pub async fn get_trust_anchor_digests(
    app: State<'_, AppStateHandle>,
) -> Result<Vec<TrustedOrganizationPublicKeyAndDigest>, CommandError> {
    let vault = app.inner().vault().await.context(VaultLockedSnafu)?;

    let now = time::now();

    let digests = vault
        .org_pks(now)
        .await
        .context(VaultSnafu {
            failed_to: "read organization public keys",
        })?
        .iter()
        .map(|org_pk| {
            TrustedOrganizationPublicKeyAndDigest::new(
                org_pk.public_key_hex(),
                human_readable_digest(&org_pk.key),
            )
        })
        .collect();

    Ok(digests)
}

#[tauri::command]
pub async fn force_rotate_id_pk(app: State<'_, AppStateHandle>) -> Result<(), CommandError> {
    let api_client = app
        .inner()
        .api_client()
        .await
        .context(ApiClientUnavailableSnafu)?;

    let vault = app.inner().vault().await.context(VaultLockedSnafu)?;

    let now = time::now();

    vault
        .generate_id_key_pair_and_rotate_pk(&api_client, now)
        .await
        .context(VaultSnafu {
            failed_to: "rotate identity key",
        })?;

    Ok(())
}

#[tauri::command]
pub async fn force_rotate_msg_pk(app: State<'_, AppStateHandle>) -> Result<(), CommandError> {
    let api_client = app
        .inner()
        .api_client()
        .await
        .context(ApiClientUnavailableSnafu)?;

    let vault = app.inner().vault().await.context(VaultLockedSnafu)?;

    let now = time::now();

    vault
        .generate_msg_key_pair_and_upload_pk(&api_client, now)
        .await
        .context(VaultSnafu {
            failed_to: "rotate messaging key",
        })?;

    Ok(())
}

#[tauri::command]
pub async fn get_public_info(app: State<'_, AppStateHandle>) -> Result<String, CommandError> {
    let public_info = app.public_info().await;
    let public_info = public_info.as_ref().context(PublicInfoUnavailableSnafu)?;

    let public_info = public_info.to_untrusted();

    let public_info_json =
        serde_json::to_string_pretty(&public_info).context(JsonSerializeSnafu)?;

    Ok(public_info_json)
}

#[tauri::command]
pub async fn get_logs(
    app: State<'_, AppStateHandle>,
) -> Result<Vec<SentinelLogEntry>, CommandError> {
    app.logs
        .get_entries()
        .await
        .map(|v| v.iter().map(SentinelLogEntry::from_log_entry).collect())
        .context(VaultSnafu {
            failed_to: "get log entries",
        })
}

#[tauri::command]
pub async fn get_vault_keys(app: State<'_, AppStateHandle>) -> Result<String, CommandError> {
    let vault = app.inner().vault().await.context(VaultLockedSnafu)?;

    let now = time::now();

    let vault_keys = vault.all_vault_keys(now).await.context(VaultSnafu {
        failed_to: "get vault keys",
    })?;

    let vault_keys_json = serde_json::to_string_pretty(&vault_keys).context(JsonSerializeSnafu)?;

    Ok(vault_keys_json)
}
