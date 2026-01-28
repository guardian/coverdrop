use common::{
    clap::Stage,
    crypto::keys::{public_key::PublicKey, untrusted::signing::UntrustedSignedPublicSigningKey},
    protocol::keys::UntrustedOrganizationPublicKey,
    protocol::{keys::AnchorOrganizationPublicKey, roles::Organization},
};
use snafu::{OptionExt as _, ResultExt as _};
use std::{collections::HashSet, path::Path};
use tauri::State;

use crate::{
    app_state::AppStateHandle,
    error::{CommandError, GenericSnafu, MissingProfileSnafu, VaultSnafu},
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
    stage: String,
    password: &str,
    app: State<'_, AppStateHandle>,
    profiles: State<'_, Profiles>,
) -> Result<OpenVaultOutcome, CommandError> {
    let profiles = profiles.inner();

    let api_url = profiles.api_url(&stage).context(MissingProfileSnafu)?;

    let stage = Stage::from_guardian_str(stage.as_str())
        .ok()
        .context(GenericSnafu {
            ctx: "No trust anchors exist for stage provide",
        })?;

    let (vault, api_client) = app
        .inner()
        .unlock_vault(stage, api_url, path, password)
        .await
        .context(VaultSnafu {
            failed_to: "unlock vault, is your password correct?",
        })?;

    let Ok(keys) = api_client.get_public_keys().await else {
        // If we're not able to get the public key hierarchy we can't
        // compare our org keys to see if theres a mismatch.
        // We perhaps ought to queue up something to run this process
        // when the internet comes back.
        return Ok(OpenVaultOutcome::OpenedOffline);
    };

    let org_pks: HashSet<UntrustedSignedPublicSigningKey<Organization>> = vault
        .org_pks()
        .context(VaultSnafu {
            failed_to: "get trust anchors for profile",
        })?
        .into_iter()
        .map(|org_pk: AnchorOrganizationPublicKey| org_pk.to_non_anchor().to_untrusted())
        .collect::<HashSet<_>>();

    let api_org_pks = keys
        .keys
        .org_pk_iter()
        .cloned()
        .collect::<HashSet<UntrustedOrganizationPublicKey>>();

    let org_pks_missing_in_api = org_pks
        .difference(&api_org_pks)
        .map(|org_pk| org_pk.public_key_hex())
        .collect();

    // TODO if the API has org keys we don't have, it means that Sentinel needs to be updated,
    // or the user has the wrong stage selected. We should lock the vault and inform the user
    // rather than proceeding with a warning.
    // https://github.com/guardian/coverdrop-internal/issues/3785
    let org_pks_missing_in_vault = api_org_pks
        .difference(&org_pks)
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
pub async fn send_notification(
    app: State<'_, AppStateHandle>,
    title: Option<String>,
    body: String,
) -> Result<(), CommandError> {
    let _ = app.notifications.send(title, body).await;
    Ok(())
}
