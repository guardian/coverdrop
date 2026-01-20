use crate::{
    app_state::AppStateHandle,
    error::{
        AnyhowSnafu, ApiClientUnavailableSnafu, CommandError, GenericSnafu, JsonSerializeSnafu,
        VaultLockedSnafu, VaultSnafu,
    },
    launch_tauri_instance,
    model::TrustedOrganizationPublicKeyAndDigest,
};
use chrono::{DateTime, Utc};
use common::{
    api::models::untrusted_keys_and_journalist_profiles::UntrustedKeysAndJournalistProfiles,
    client::JournalistStatus,
    crypto::{human_readable_digest, keys::public_key::PublicKey as _},
    time,
};
use journalist_vault::logging::{LogEntry, LoggingSession};
use snafu::{OptionExt as _, ResultExt as _};
use tauri::State;

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
pub async fn get_public_info(
    app: State<'_, AppStateHandle>,
) -> Result<Option<UntrustedKeysAndJournalistProfiles>, CommandError> {
    let public_info = app.public_info().await;
    let public_info = public_info.as_ref();
    // public_info may be None if the initial run of the task hasn't completed
    match public_info {
        Some(public_info) => Ok(Some(public_info.to_untrusted())),
        None => Ok(None),
    }
}

#[tauri::command]
pub async fn get_logging_sessions_timeline(
    app: State<'_, AppStateHandle>,
) -> Result<Vec<LoggingSession>, CommandError> {
    app.logs.get_sessions_timeline().await.context(VaultSnafu {
        failed_to: "get logging sessions",
    })
}

#[tauri::command]
pub async fn get_logs(
    app: State<'_, AppStateHandle>,
    min_level: String,
    search_term: String,
    before: DateTime<Utc>,
    limit: i64,
    offset: i64,
) -> Result<Vec<LogEntry>, CommandError> {
    app.logs
        .get_entries(min_level, search_term, before, limit, offset)
        .await
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

#[tauri::command]
pub async fn update_journalist_status(
    app: State<'_, AppStateHandle>,
    new_status: JournalistStatus,
) -> Result<(), CommandError> {
    let new_status = new_status.clone();

    let api_client = app
        .inner()
        .api_client()
        .await
        .context(ApiClientUnavailableSnafu)?;

    let vault = app.inner().vault().await.context(VaultLockedSnafu)?;
    let journalist_id = vault.journalist_id().await.context(VaultSnafu {
        failed_to: "get latest identity key pair",
    })?;

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

    api_client
        .patch_journalist_status(journalist_id, &latest_id_key_pair, new_status, now)
        .await
        .context(AnyhowSnafu {
            failed_to: "send patch request to API",
        })?;

    Ok(())
}

#[tauri::command]
pub async fn launch_new_instance() -> Result<(), CommandError> {
    // TODO ideally make this a signal to parent process to start this, so all child processes descend from the daemon
    launch_tauri_instance()
        .wait()
        .expect("waiting on new instance");

    Ok(())
}

#[tauri::command]
pub async fn fully_exit_app() -> Result<(), CommandError> {
    std::process::exit(1)
}
