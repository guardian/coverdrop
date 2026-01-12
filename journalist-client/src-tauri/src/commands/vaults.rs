use common::{
    crypto::keys::{public_key::PublicKey, serde::StorableKeyMaterial},
    protocol::keys::{anchor_org_pk, UntrustedOrganizationPublicKey},
    time,
};
use snafu::{OptionExt as _, ResultExt as _};
use std::{collections::HashSet, path::Path};
use tauri::State;

use crate::{
    app_state::AppStateHandle,
    error::{
        AnyhowSnafu, CommandError, GenericSnafu, MissingProfileSnafu, VaultLockedSnafu, VaultSnafu,
    },
    model::{OpenVaultOutcome, Profiles, VaultState},
};

#[tauri::command]
pub async fn get_vault_state(
    app: State<'_, AppStateHandle>,
) -> Result<Option<VaultState>, CommandError> {
    app.inner().vault_state().await.context(VaultSnafu {
        failed_to: "get vault state",
    })
}

#[tauri::command]
pub async fn unlock_vault(
    path: &Path,
    profile: &str,
    password: &str,
    app: State<'_, AppStateHandle>,
    profiles: State<'_, Profiles>,
) -> Result<OpenVaultOutcome, CommandError> {
    let profiles = profiles.inner();

    let api_url = profiles.api_url(profile).context(MissingProfileSnafu)?;

    let (vault, api_client) = app
        .inner()
        .unlock_vault(api_url, path, password)
        .await
        .context(VaultSnafu {
            failed_to: "unlock vault, is your password correct?",
        })?;

    let now = time::now();

    let Ok(keys) = api_client.get_public_keys().await else {
        // If we're not able to get the public key hierarchy we can't
        // compare our org keys to see if theres a mismatch.
        // We perhaps ought to queue up something to run this process
        // when the internet comes back.
        return Ok(OpenVaultOutcome::OpenedOffline);
    };

    // Failure to read the org pks out of the DB is a bad sign so let's just fail opening if we can't
    let vault_org_pks = vault
        .org_pks(now)
        .await
        .context(VaultSnafu {
            failed_to: "read trust anchors from vault",
        })?
        .iter()
        .map(|org_pk| org_pk.to_non_anchor().to_untrusted())
        .collect::<HashSet<_>>();

    let api_org_pks = keys
        .keys
        .org_pk_iter()
        .cloned()
        .collect::<HashSet<UntrustedOrganizationPublicKey>>();

    let org_pks_missing_in_api = vault_org_pks
        .difference(&api_org_pks)
        .map(|org_pk| org_pk.public_key_hex())
        .collect();

    let org_pks_missing_in_vault = api_org_pks
        .difference(&vault_org_pks)
        .map(|org_pk| org_pk.public_key_hex())
        .collect();

    Ok(OpenVaultOutcome::OpenedOnline {
        org_pks_missing_in_vault,
        org_pks_missing_in_api,
    })
}

#[tauri::command]
pub async fn soft_lock_vault(
    app: State<'_, AppStateHandle>,
) -> Result<Option<VaultState>, CommandError> {
    app.inner().soft_lock_vault().await.context(VaultSnafu {
        failed_to: "soft lock vault",
    })
}

#[tauri::command]
pub async fn unlock_soft_locked_vault(
    app: State<'_, AppStateHandle>,
    password: String,
) -> Result<Option<VaultState>, CommandError> {
    app.inner()
        .unlock_soft_locked_vault(&password)
        .await
        .context(VaultSnafu {
            failed_to: "unlock soft locked vault",
        })
}

#[tauri::command]
pub async fn get_colocated_password(path: &Path) -> Result<Option<String>, CommandError> {
    let password_path = path.with_extension("password");

    let password_result = std::fs::read_to_string(&password_path);

    match password_result {
        Ok(password) => {
            if password.contains('\n') {
                return Err(GenericSnafu {
                    ctx: "Password file found, but it was not a single line",
                }
                .build());
            }

            Ok(Some(password))
        }
        Err(_) => Ok(None),
    }
}

#[tauri::command]
pub async fn add_trust_anchor(
    path: &Path,
    app: State<'_, AppStateHandle>,
) -> Result<(), CommandError> {
    let vault = app.inner().vault().await.context(VaultLockedSnafu)?;

    // We skip the permissions check here since that is designed for use on services which treat a badly permissioned
    // key as an invalid state and panic. We don't expect users of the journalist client to even understand what unix
    // permissions are, never mind set them. Given this will only be used when loading public keys I think this is alright.
    let org_pk = UntrustedOrganizationPublicKey::load_from_file_skip_permissions_check(path)
        .context(AnyhowSnafu {
            failed_to: "read new trust anchor from disk",
        })?;

    let now = time::now();

    let org_pk = anchor_org_pk(&org_pk.to_tofu_anchor(), now).context(AnyhowSnafu {
        failed_to: "verify new trust anchor",
    })?;

    vault.add_org_pk(&org_pk, now).await.context(VaultSnafu {
        failed_to: "add new anchor",
    })?;

    Ok(())
}

#[tauri::command]
pub async fn send_notification(
    app: State<'_, AppStateHandle>,
    title: Option<String>,
    body: String,
) -> Result<(), CommandError> {
    let _ = app.notifications.send(title, body).await;
    Ok(())
}
