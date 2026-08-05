use crate::{
    anchor_org_pk_cache::AnchorOrganizationPublicKeyCache,
    cache_control::{add_cache_control_header, PUBLIC_KEYS_TTL, ROTATION_FORM_TTL},
    constants::MAX_NON_DESK_JOURNALIST_DESCRIPTION_LEN,
    error::AppError,
    services::database::Database,
};
use axum::extract::{Path, State};
use axum::Json;
use chrono::{DateTime, Utc};
use common::api::{
    constants::{HEADER_SENTINEL_APP_NAME, HEADER_SENTINEL_BUILT_AT, HEADER_SENTINEL_GIT_SHA},
    models::sentinel_id_and_id_pk_rotation_form::SentinelIdAndPublicKeyRotationForm,
};
use common::{
    api::{
        forms::{
            DeleteJournalistForm, PatchJournalistForm, PostAdminPublicKeyForm,
            PostCoverNodeIdPublicKeyBody, PostCoverNodeIdPublicKeyForm,
            PostCoverNodeMessagingPublicKeyForm, PostCoverNodeProvisioningPublicKeyForm,
            PostJournalistForm, PostJournalistIdPublicKeyBody, PostJournalistIdPublicKeyForm,
            PostJournalistMessagingPublicKeyForm, PostJournalistProvisioningPublicKeyForm,
            PostSentinelIdPublicKeyBody, PostSentinelIdPublicKeyForm, PostSentinelProfileForm,
            RotateJournalistIdPublicKeyFormBody, RotateJournalistIdPublicKeyFormForm,
            RotateSentinelIdPublicKeyFormBody, RotateSentinelIdPublicKeyFormForm,
        },
        models::{
            journalist_id::JournalistIdentity,
            journalist_id_and_id_pk_rotation_form::JournalistIdAndPublicKeyRotationForm,
            untrusted_keys_and_journalist_profiles::UntrustedKeysAndJournalistProfiles,
        },
    },
    crypto::keys::role::Role,
    epoch::Epoch,
    identity_api::{
        forms::post_rotate_journalist_id::RotateJournalistIdPublicKeyBody,
        forms::post_rotate_sentinel_id::RotateSentinelIdPublicKeyBody,
        models::UntrustedJournalistIdPublicKeyWithEpoch,
        models::UntrustedSentinelIdPublicKeyWithEpoch,
    },
    protocol::{
        constants::{
            COVERNODE_ID_KEY_ROTATE_AFTER, COVERNODE_MSG_KEY_ROTATE_AFTER,
            COVERNODE_PROVISIONING_KEY_ROTATE_AFTER, JOURNALIST_ID_KEY_ROTATE_AFTER,
            JOURNALIST_MSG_KEY_ROTATE_AFTER, JOURNALIST_PROVISIONING_KEY_ROTATE_AFTER,
            SENTINEL_ID_KEY_ROTATE_AFTER,
        },
        keys::{
            verify_covernode_id_pk, verify_covernode_messaging_pk,
            verify_covernode_provisioning_pk, verify_journalist_id_pk,
            verify_journalist_messaging_pk, verify_journalist_provisioning_pk,
            verify_sentinel_id_pk,
        },
        roles::{CoverNodeId, CoverNodeMessaging, JournalistId, JournalistMessaging, SentinelId},
    },
    system::keys::verify_admin_pk,
    time,
};
use http::HeaderMap;

pub async fn get_public_keys(
    State(anchor_org_pks): State<AnchorOrganizationPublicKeyCache>,
    State(db): State<Database>,
    State(default_journalist_id): State<Option<JournalistIdentity>>,
) -> Result<(HeaderMap, Json<UntrustedKeysAndJournalistProfiles>), AppError> {
    let (keys, max_epoch) = {
        let anchor_org_pks = anchor_org_pks.get().await;
        let (key, max_epoch) = db
            .hierarchy_queries
            .key_hierarchy(&anchor_org_pks, time::now())
            .await?;

        (key, Epoch(max_epoch))
    };

    let journalist_profiles = db.journalist_queries.journalist_profiles().await?;
    let sentinel_profiles = db.sentinel_queries.sentinel_profiles().await?;

    let default_journalist_id = default_journalist_id.filter(|default_journalist_id| {
        keys.journalist_id_iter()
            .any(|existing_journalist_id| existing_journalist_id == default_journalist_id)
    });

    let keys = keys.to_untrusted();

    let mut headers = HeaderMap::new();
    add_cache_control_header(&mut headers, PUBLIC_KEYS_TTL);

    Ok((
        headers,
        Json(UntrustedKeysAndJournalistProfiles::new(
            journalist_profiles,
            default_journalist_id,
            keys,
            max_epoch,
            sentinel_profiles,
        )),
    ))
}

pub async fn post_journalist(
    State(anchor_org_pks): State<AnchorOrganizationPublicKeyCache>,
    State(db): State<Database>,
    Json(form): Json<PostJournalistForm>,
) -> Result<(), AppError> {
    let (keys, _max_epoch) = db
        .hierarchy_queries
        .key_hierarchy(&anchor_org_pks.get().await, time::now())
        .await?;

    let verifying_provisioning_pk = keys
        .find_journalist_provisioning_pk_from_raw_ed25519_pk(form.signing_pk())
        .ok_or(AppError::SigningKeyNotFound)?;

    let body = form
        .to_verified_form_data(verifying_provisioning_pk, time::now())
        .map_err(|e| {
            tracing::error!("Failed to verify form {}", e);
            AppError::SignatureVerificationFailed
        })?;

    if !body.is_desk {
        // Journalist descriptions can't be too long
        if body.description.len() > MAX_NON_DESK_JOURNALIST_DESCRIPTION_LEN {
            return Err(AppError::JournalistDescriptionTooLong);
        }
    }

    db.journalist_queries
        .insert_journalist_profile(body, time::now())
        .await?;

    Ok(())
}

pub async fn post_sentinel_profile(
    State(anchor_org_pks): State<AnchorOrganizationPublicKeyCache>,
    State(db): State<Database>,
    Json(form): Json<PostSentinelProfileForm>,
) -> Result<(), AppError> {
    let (keys, _max_epoch) = db
        .hierarchy_queries
        .key_hierarchy(&anchor_org_pks.get().await, time::now())
        .await?;

    let verifying_provisioning_pk = keys
        .find_journalist_provisioning_pk_from_raw_ed25519_pk(form.signing_pk())
        .ok_or(AppError::SigningKeyNotFound)?;

    let body = form
        .to_verified_form_data(verifying_provisioning_pk, time::now())
        .map_err(|e| {
            tracing::error!("Failed to verify form {}", e);
            AppError::SignatureVerificationFailed
        })?;

    db.sentinel_queries
        .insert_sentinel_profile(body.id, body.display_name, time::now())
        .await?;

    Ok(())
}

pub async fn patch_journalist(
    State(anchor_org_pks): State<AnchorOrganizationPublicKeyCache>,
    State(db): State<Database>,
    Json(form): Json<PatchJournalistForm>,
) -> Result<(), AppError> {
    let (keys, _max_epoch) = db
        .hierarchy_queries
        .key_hierarchy(&anchor_org_pks.get().await, time::now())
        .await?;

    let verifying_provisioning_pk = keys
        .find_journalist_provisioning_pk_from_raw_ed25519_pk(form.signing_pk())
        .ok_or(AppError::SigningKeyNotFound)?;

    let body = form
        .to_verified_form_data(verifying_provisioning_pk, time::now())
        .map_err(|e| {
            tracing::error!("Failed to verify form {}", e);
            AppError::SignatureVerificationFailed
        })?;

    db.journalist_queries
        .update_journalist_profile(
            body.journalist_id,
            body.display_name,
            body.sort_name,
            body.is_desk,
            body.description,
        )
        .await?;

    Ok(())
}

pub async fn delete_journalist(
    State(anchor_org_pks): State<AnchorOrganizationPublicKeyCache>,
    State(db): State<Database>,
    Json(form): Json<DeleteJournalistForm>,
) -> Result<(), AppError> {
    let (keys, _max_epoch) = db
        .hierarchy_queries
        .key_hierarchy(&anchor_org_pks.get().await, time::now())
        .await?;

    let verifying_provisioning_pk = keys
        .find_journalist_provisioning_pk_from_raw_ed25519_pk(form.signing_pk())
        .ok_or(AppError::SigningKeyNotFound)?;

    let journalist_id = form
        .to_verified_form_data(verifying_provisioning_pk, time::now())
        .map_err(|e| {
            tracing::error!("Failed to verify form {}", e);
            AppError::SignatureVerificationFailed
        })?;

    db.journalist_queries
        .delete_journalist(&journalist_id)
        .await?;

    Ok(())
}

/// Function used by key upload controllers to check if a key has been rotated too recently
fn check_if_key_rotation_too_recent(
    latest_pk_added_at: Option<DateTime<Utc>>,
    min_rotate_after: chrono::Duration,
) -> Result<(), AppError> {
    if let Some(latest_added_at) = latest_pk_added_at {
        let duration_since_last_rotation = (time::now() - latest_added_at).abs();
        if duration_since_last_rotation < min_rotate_after {
            return Err(AppError::KeyRotationTooRecent);
        }
    }

    Ok(())
}

/// Function used by key upload controllers to warn if a key has been rotated too recently
/// This will be useful if the system gets stuck in a state where it is consistently rotating
fn warn_if_key_rotation_too_recent<R: Role>(
    latest_pk_added_at: Option<DateTime<Utc>>,
    min_rotate_after: chrono::Duration,
) {
    if let Some(latest_added_at) = latest_pk_added_at {
        let duration_since_last_rotation = (time::now() - latest_added_at).abs();
        if duration_since_last_rotation < min_rotate_after {
            tracing::warn!("Key {} has been rotated too recently", R::display());
        }
    }
}

pub async fn post_covernode_provisioning_key(
    State(anchor_org_pks): State<AnchorOrganizationPublicKeyCache>,
    State(db): State<Database>,
    Json(form): Json<PostCoverNodeProvisioningPublicKeyForm>,
) -> Result<(), AppError> {
    let (keys, _max_epoch) = db
        .hierarchy_queries
        .key_hierarchy(&anchor_org_pks.get().await, time::now())
        .await?;

    let verifying_org_pk = keys
        .find_org_pk_from_raw_ed25519_pk(form.signing_pk())
        .ok_or(AppError::SigningKeyNotFound)?;

    let latest_pk_added_at = db
        .covernode_key_queries
        .latest_provisioning_pk_added_at()
        .await?;

    check_if_key_rotation_too_recent(latest_pk_added_at, COVERNODE_PROVISIONING_KEY_ROTATE_AFTER)?;

    let new_provisioning_pk = form
        .to_verified_form_data(verifying_org_pk, time::now())
        .map_err(|e| {
            tracing::error!("Failed to verify form {}", e);
            AppError::SignatureVerificationFailed
        })?;

    let new_provisioning_pk =
        verify_covernode_provisioning_pk(&new_provisioning_pk, verifying_org_pk, time::now())
            .map_err(|e| {
                tracing::error!("Failed to verify covernode provisioning key {}", e);
                AppError::SignatureVerificationFailed
            })?;

    db.covernode_key_queries
        .insert_covernode_provisioning_pk(&new_provisioning_pk, verifying_org_pk, time::now())
        .await?;

    metrics::counter!("CoverNodeProvisioningPksAdded").increment(1);

    Ok(())
}

pub async fn post_covernode_id_key(
    State(anchor_org_pks): State<AnchorOrganizationPublicKeyCache>,
    State(db): State<Database>,
    Json(form): Json<PostCoverNodeIdPublicKeyForm>,
) -> Result<Json<Epoch>, AppError> {
    let (keys, _max_epoch) = db
        .hierarchy_queries
        .key_hierarchy(&anchor_org_pks.get().await, time::now())
        .await?;

    let form_signing_provisioning_pk = keys
        .find_covernode_provisioning_pk_from_raw_ed25519_pk(form.signing_pk())
        .ok_or(AppError::SigningKeyNotFound)?;

    let PostCoverNodeIdPublicKeyBody {
        covernode_id,
        covernode_id_pk,
    } = form
        .to_verified_form_data(form_signing_provisioning_pk, time::now())
        .map_err(|e| {
            tracing::error!("Failed to verify form {}", e);
            AppError::SignatureVerificationFailed
        })?;

    let (new_id_pk, key_signing_provisioning_pk) = keys
        .covernode_provisioning_pk_iter()
        .find_map(|covernode_provisioning_pk| {
            match verify_covernode_id_pk(&covernode_id_pk, covernode_provisioning_pk, time::now()) {
                Ok(id_pk) => Some((id_pk, covernode_provisioning_pk)),
                Err(_) => None,
            }
        })
        .ok_or_else(|| {
            tracing::error!("Failed to verify covernode id key");
            AppError::SignatureVerificationFailed
        })?;

    // It should be a rare edge case that the covernode provisioning key rotated between the time that the id key was created and published.
    if key_signing_provisioning_pk != form_signing_provisioning_pk {
        tracing::error!("Covernode id key was not signed by the expected provisioning key");
    }

    let latest_pk_added_at = db
        .covernode_key_queries
        .latest_id_pk_added_at(&covernode_id)
        .await?;

    warn_if_key_rotation_too_recent::<CoverNodeId>(
        latest_pk_added_at,
        COVERNODE_ID_KEY_ROTATE_AFTER,
    );

    let epoch = db
        .covernode_key_queries
        .insert_covernode_id_pk(
            &covernode_id,
            &new_id_pk,
            key_signing_provisioning_pk,
            time::now(),
        )
        .await?;

    Ok(Json(epoch))
}

pub async fn post_covernode_msg_key(
    State(anchor_org_pks): State<AnchorOrganizationPublicKeyCache>,
    State(db): State<Database>,
    Json(form): Json<PostCoverNodeMessagingPublicKeyForm>,
) -> Result<Json<Epoch>, AppError> {
    let (keys, _max_epoch) = db
        .hierarchy_queries
        .key_hierarchy(&anchor_org_pks.get().await, time::now())
        .await?;

    let (covernode_id, form_signing_id_pk) = keys
        .find_covernode_id_pk_from_raw_ed25519_pk(form.signing_pk())
        .ok_or(AppError::SigningKeyNotFound)?;

    let covernode_msg_pk = form
        .to_verified_form_data(form_signing_id_pk, time::now())
        .map_err(|e| {
            tracing::error!("Failed to verify form {}", e);
            AppError::SignatureVerificationFailed
        })?;

    let (new_msg_pk, key_signing_id_pk) = keys
        .covernode_id_pk_iter_for_identity(covernode_id)
        .find_map(|covernode_id_pk| {
            match verify_covernode_messaging_pk(&covernode_msg_pk, covernode_id_pk, time::now()) {
                Ok(msg_pk) => Some((msg_pk, covernode_id_pk)),
                Err(_) => None,
            }
        })
        .ok_or_else(|| {
            tracing::error!("Failed to verify covernode msg key");
            AppError::SignatureVerificationFailed
        })?;

    // It should be a rare edge case that covernode id key rotated between the time that the msg key was created and published.
    if key_signing_id_pk != form_signing_id_pk {
        tracing::error!("Covernode msg key was not signed by the expected id key");
    }

    let latest_pk_added_at = db
        .covernode_key_queries
        .latest_msg_pk_added_at(covernode_id)
        .await?;

    warn_if_key_rotation_too_recent::<CoverNodeMessaging>(
        latest_pk_added_at,
        COVERNODE_MSG_KEY_ROTATE_AFTER,
    );

    let epoch = db
        .covernode_key_queries
        .insert_covernode_msg_pk(covernode_id, &new_msg_pk, key_signing_id_pk, time::now())
        .await?;

    Ok(Json(epoch))
}

pub async fn post_journalist_provisioning_key(
    State(anchor_org_pks): State<AnchorOrganizationPublicKeyCache>,
    State(db): State<Database>,
    Json(form): Json<PostJournalistProvisioningPublicKeyForm>,
) -> Result<(), AppError> {
    let (keys, _max_epoch) = db
        .hierarchy_queries
        .key_hierarchy(&anchor_org_pks.get().await, time::now())
        .await?;

    let verifying_org_pk = keys
        .find_org_pk_from_raw_ed25519_pk(form.signing_pk())
        .ok_or(AppError::SigningKeyNotFound)?;

    let latest_pk_added_at = db
        .journalist_queries
        .latest_provisioning_pk_added_at()
        .await?;

    check_if_key_rotation_too_recent(latest_pk_added_at, JOURNALIST_PROVISIONING_KEY_ROTATE_AFTER)?;

    let new_provisioning_pk = form
        .to_verified_form_data(verifying_org_pk, time::now())
        .map_err(|e| {
            tracing::error!("Failed to verify form {}", e);
            AppError::SignatureVerificationFailed
        })?;

    let new_provisioning_pk =
        verify_journalist_provisioning_pk(&new_provisioning_pk, verifying_org_pk, time::now())
            .map_err(|e| {
                tracing::error!("Failed to verify journalist provisioning key {}", e);
                AppError::SignatureVerificationFailed
            })?;

    db.journalist_queries
        .insert_journalist_provisioning_pk(&new_provisioning_pk, verifying_org_pk, time::now())
        .await?;

    metrics::counter!("JournalistProvisioningPksAdded").increment(1);

    Ok(())
}

pub async fn post_journalist_id_pk_rotation_form(
    State(anchor_org_pks): State<AnchorOrganizationPublicKeyCache>,
    State(db): State<Database>,
    Json(form): Json<RotateJournalistIdPublicKeyFormForm>,
) -> Result<(), AppError> {
    let (keys, _max_epoch) = db
        .hierarchy_queries
        .key_hierarchy(&anchor_org_pks.get().await, time::now())
        .await?;

    let (journalist_id, verifying_id_pk) = keys
        .find_journalist_id_pk_from_raw_ed25519_pk(form.signing_pk())
        .ok_or(AppError::SigningKeyNotFound)?;

    // Unwrap the outer form by verifying the signature
    let RotateJournalistIdPublicKeyFormBody { form } = form
        .to_verified_form_data(verifying_id_pk, time::now())
        .map_err(|e| {
            tracing::error!("Failed to verify form {}", e);
            AppError::SignatureVerificationFailed
        })?;

    // Verify and read out the inner form's public key. Used to run soundness checks
    // such as checking if the key has already been published.
    let RotateJournalistIdPublicKeyBody { new_pk } = form
        .to_verified_form_data(verifying_id_pk, time::now())
        .map_err(|e| {
            tracing::error!("Failed to verify form {}", e);
            AppError::SignatureVerificationFailed
        })?;

    db.journalist_queries
        .insert_journalist_id_pk_rotation_form(journalist_id, &form, &new_pk)
        .await?;

    Ok(())
}

pub async fn get_journalist_id_pk_rotation_forms(
    State(db): State<Database>,
) -> Result<(HeaderMap, Json<Vec<JournalistIdAndPublicKeyRotationForm>>), AppError> {
    let result = db
        .journalist_queries
        .select_journalist_id_pk_rotation_forms(time::now())
        .await
        .map_err(|e| {
            tracing::error!("Failed to select journalist ID pk rotation forms: {:?}", e);
            AppError::Anyhow(e)
        })?;

    let mut headers = HeaderMap::new();
    add_cache_control_header(&mut headers, ROTATION_FORM_TTL);

    Ok((headers, Json(result)))
}

/// Upload a new journalist ID key that has been signed using a journalist provisioning key, either
/// by the on-premises identity services or by an admin.
pub async fn post_journalist_id_key(
    State(anchor_org_pks): State<AnchorOrganizationPublicKeyCache>,
    State(db): State<Database>,
    Json(form): Json<PostJournalistIdPublicKeyForm>,
) -> Result<Json<Epoch>, AppError> {
    let (keys, _max_epoch) = db
        .hierarchy_queries
        .key_hierarchy(&anchor_org_pks.get().await, time::now())
        .await?;

    let form_signing_provisioning_pk = keys
        .find_journalist_provisioning_pk_from_raw_ed25519_pk(form.signing_pk())
        .ok_or(AppError::SigningKeyNotFound)?;

    let PostJournalistIdPublicKeyBody {
        journalist_id,
        journalist_id_pk,
        from_queue,
    } = form
        .to_verified_form_data(form_signing_provisioning_pk, time::now())
        .map_err(|e| {
            tracing::error!("Failed to verify form {}", e);
            AppError::SignatureVerificationFailed
        })?;

    let (new_id_pk, key_signing_provisioning_pk) = keys
        .journalist_provisioning_pk_iter()
        .find_map(|journalist_provisioning_pk| {
            match verify_journalist_id_pk(
                &journalist_id_pk,
                journalist_provisioning_pk,
                time::now(),
            ) {
                Ok(id_pk) => Some((id_pk, journalist_provisioning_pk)),
                Err(_) => None,
            }
        })
        .ok_or_else(|| {
            tracing::error!("Failed to verify journalist id key");
            AppError::SignatureVerificationFailed
        })?;

    // It should be a rare edge case that journalist provisioning key rotated between the time that the id key was created and published.
    if key_signing_provisioning_pk != form_signing_provisioning_pk {
        tracing::error!("Journalist id key was not signed by the expected provisioning key");
    }

    let latest_pk_added_at = db
        .journalist_queries
        .latest_id_pk_added_at(&journalist_id)
        .await?;

    warn_if_key_rotation_too_recent::<JournalistId>(
        latest_pk_added_at,
        JOURNALIST_ID_KEY_ROTATE_AFTER,
    );

    let epoch = db
        .journalist_queries
        .insert_journalist_id_pk(
            &journalist_id,
            &new_id_pk,
            from_queue,
            key_signing_provisioning_pk,
            time::now(),
        )
        .await?;

    Ok(Json(epoch))
}

/// Unauthenticated endpoint used by Sentinel to find out whether a given candidate journalist ID PK has been published.
pub async fn get_journalist_id_pk_with_epoch(
    State(db): State<Database>,
    Path(pk_hex): Path<String>,
) -> Result<
    (
        HeaderMap,
        Json<Option<UntrustedJournalistIdPublicKeyWithEpoch>>,
    ),
    AppError,
> {
    let pk_with_epoch = db
        .journalist_queries
        .get_journalist_id_pk_with_epoch_from_ed25519_pk(&pk_hex)
        .await
        .map_err(|e| {
            tracing::error!("Failed to read journalist ID pk from database: {:?}", e);
            AppError::Anyhow(e)
        })?;

    let mut headers = HeaderMap::new();
    add_cache_control_header(&mut headers, PUBLIC_KEYS_TTL);

    Ok((headers, Json(pk_with_epoch)))
}

pub async fn post_journalist_msg_key(
    headers: HeaderMap,
    State(anchor_org_pks): State<AnchorOrganizationPublicKeyCache>,
    State(db): State<Database>,
    Json(form): Json<PostJournalistMessagingPublicKeyForm>,
) -> Result<Json<Epoch>, AppError> {
    let (keys, _max_epoch) = db
        .hierarchy_queries
        .key_hierarchy(&anchor_org_pks.get().await, time::now())
        .await?;

    let (journalist_id, form_signing_id_pk) = keys
        .find_journalist_id_pk_from_raw_ed25519_pk(form.signing_pk())
        .ok_or(AppError::SigningKeyNotFound)?;

    let header_as_string = |header: &str| -> String {
        headers
            .get(header)
            .map_or_else(|| "", |v| v.to_str().unwrap_or("Invalid UTF-8"))
            .to_string()
    };

    tracing::info!(
        journalistId = journalist_id.to_string(),
        sentinelAppName = header_as_string(HEADER_SENTINEL_APP_NAME),
        sentinelGitSHA = header_as_string(HEADER_SENTINEL_GIT_SHA),
        sentinelBuiltAt = header_as_string(HEADER_SENTINEL_BUILT_AT),
        "Attempting journalist message key rotation"
    );

    // Check the form is valid
    let new_msg_pk = form
        .to_verified_form_data(form_signing_id_pk, time::now())
        .map_err(|e| {
            tracing::error!("Failed to verify form {}", e);
            AppError::SignatureVerificationFailed
        })?;

    let (new_msg_pk, key_signing_id_pk) = keys
        .journalist_id_pk_iter_for_identity(journalist_id)
        .find_map(|journalist_id_pk| {
            match verify_journalist_messaging_pk(&new_msg_pk, journalist_id_pk, time::now()) {
                Ok(msg_pk) => Some((msg_pk, journalist_id_pk)),
                Err(_) => None,
            }
        })
        .ok_or_else(|| {
            tracing::error!("Failed to verify journalist msg key");
            AppError::SignatureVerificationFailed
        })?;

    // It should be a rare edge case that journalist id key rotated between the time that the msg key was created and published.
    if key_signing_id_pk != form_signing_id_pk {
        tracing::error!("Journalist msg key was not signed by the expected id key");
    }

    let latest_pk_added_at = db
        .journalist_queries
        .latest_msg_pk_added_at(journalist_id)
        .await?;

    warn_if_key_rotation_too_recent::<JournalistMessaging>(
        latest_pk_added_at,
        JOURNALIST_MSG_KEY_ROTATE_AFTER,
    );

    let epoch = db
        .journalist_queries
        .insert_journalist_msg_pk(journalist_id, new_msg_pk, key_signing_id_pk, time::now())
        .await?;

    Ok(Json(epoch))
}

pub async fn post_admin_key(
    State(anchor_org_pks): State<AnchorOrganizationPublicKeyCache>,
    State(db): State<Database>,
    Json(form): Json<PostAdminPublicKeyForm>,
) -> Result<(), AppError> {
    let (keys, _max_epoch) = db
        .hierarchy_queries
        .key_hierarchy(&anchor_org_pks.get().await, time::now())
        .await?;

    let verifying_org_pk = keys
        .find_org_pk_from_raw_ed25519_pk(form.signing_pk())
        .ok_or(AppError::SigningKeyNotFound)?;

    let admin_pk = form
        .to_verified_form_data(verifying_org_pk, time::now())
        .map_err(|e| {
            tracing::error!("Failed to verify form {}", e);
            AppError::SignatureVerificationFailed
        })?;

    let admin_pk = verify_admin_pk(&admin_pk, verifying_org_pk, time::now()).map_err(|e| {
        tracing::error!("Failed to verify admin key {}", e);
        AppError::SignatureVerificationFailed
    })?;

    db.system_key_queries
        .insert_admin_pk(&admin_pk, verifying_org_pk)
        .await?;

    Ok(())
}

/// Handles post request from Sentinel when rotating sentinel ID key
pub async fn post_sentinel_id_pk_rotation_form(
    State(anchor_org_pks): State<AnchorOrganizationPublicKeyCache>,
    State(db): State<Database>,
    Json(form): Json<RotateSentinelIdPublicKeyFormForm>,
) -> Result<(), AppError> {
    let (keys, _max_epoch) = db
        .hierarchy_queries
        .key_hierarchy(&anchor_org_pks.get().await, time::now())
        .await?;

    let (sentinel_id, verifying_id_pk) = keys
        .find_sentinel_id_pk_from_raw_ed25519_pk(form.signing_pk())
        .ok_or(AppError::SigningKeyNotFound)?;

    // Unwrap the outer form by verifying the signature
    let RotateSentinelIdPublicKeyFormBody { form } = form
        .to_verified_form_data(verifying_id_pk, time::now())
        .map_err(|e| {
            tracing::error!("Failed to verify form {}", e);
            AppError::SignatureVerificationFailed
        })?;

    // Verify and read out the inner form's public key. Used to run soundness checks
    // such as checking if the key has already been published.
    let RotateSentinelIdPublicKeyBody { new_pk } = form
        .to_verified_form_data(verifying_id_pk, time::now())
        .map_err(|e| {
            tracing::error!("Failed to verify form {}", e);
            AppError::SignatureVerificationFailed
        })?;

    db.sentinel_queries
        .insert_sentinel_id_pk_rotation_form(sentinel_id, &form, &new_pk)
        .await?;

    Ok(())
}

/// Unauthenticated endpoint used by the Identity API to get forms containing sentinel ID PKs to sign
pub async fn get_sentinel_id_pk_rotation_forms(
    State(db): State<Database>,
) -> Result<(HeaderMap, Json<Vec<SentinelIdAndPublicKeyRotationForm>>), AppError> {
    let result = db
        .sentinel_queries
        .select_sentinel_id_pk_rotation_forms(time::now())
        .await
        .map_err(|e| {
            tracing::error!("Failed to select sentinel ID pk rotation forms: {:?}", e);
            AppError::Anyhow(e)
        })?;

    let mut headers = HeaderMap::new();
    add_cache_control_header(&mut headers, ROTATION_FORM_TTL);

    Ok((headers, Json(result)))
}

/// Upload a new sentinel ID key that has been signed with a journalist provisioning key, either
/// by the on-premises Identity API or by an admin.
pub async fn post_sentinel_id_key(
    State(anchor_org_pks): State<AnchorOrganizationPublicKeyCache>,
    State(db): State<Database>,
    Json(form): Json<PostSentinelIdPublicKeyForm>,
) -> Result<Json<Epoch>, AppError> {
    let (keys, _max_epoch) = db
        .hierarchy_queries
        .key_hierarchy(&anchor_org_pks.get().await, time::now())
        .await?;

    let form_signing_provisioning_pk = keys
        .find_journalist_provisioning_pk_from_raw_ed25519_pk(form.signing_pk())
        .ok_or(AppError::SigningKeyNotFound)?;

    let PostSentinelIdPublicKeyBody {
        sentinel_id,
        sentinel_id_pk,
        from_queue,
    } = form
        .to_verified_form_data(form_signing_provisioning_pk, time::now())
        .map_err(|e| {
            tracing::error!("Failed to verify form {}", e);
            AppError::SignatureVerificationFailed
        })?;

    let (new_id_pk, key_signing_provisioning_pk) = keys
        .journalist_provisioning_pk_iter()
        .find_map(|journalist_provisioning_pk| {
            match verify_sentinel_id_pk(&sentinel_id_pk, journalist_provisioning_pk, time::now()) {
                Ok(id_pk) => Some((id_pk, journalist_provisioning_pk)),
                Err(_) => None,
            }
        })
        .ok_or_else(|| {
            tracing::error!("Failed to verify sentinel id key");
            AppError::SignatureVerificationFailed
        })?;

    if key_signing_provisioning_pk != form_signing_provisioning_pk {
        tracing::error!("Sentinel id key was not signed by the expected provisioning key");
    }

    let latest_pk_added_at = db
        .sentinel_queries
        .latest_id_pk_added_at(&sentinel_id)
        .await?;

    warn_if_key_rotation_too_recent::<SentinelId>(latest_pk_added_at, SENTINEL_ID_KEY_ROTATE_AFTER);

    let epoch = db
        .sentinel_queries
        .insert_sentinel_id_pk(
            &sentinel_id,
            &new_id_pk,
            from_queue,
            key_signing_provisioning_pk,
            time::now(),
        )
        .await?;

    Ok(Json(epoch))
}

/// Unauthenticated endpoint used by Sentinel to find out whether a given candidate Sentinel ID PK has been published.
pub async fn get_sentinel_id_pk_with_epoch(
    State(db): State<Database>,
    Path(pk_hex): Path<String>,
) -> Result<
    (
        HeaderMap,
        Json<Option<UntrustedSentinelIdPublicKeyWithEpoch>>,
    ),
    AppError,
> {
    let pk_with_epoch = db
        .sentinel_queries
        .get_sentinel_id_pk_with_epoch_from_ed25519_pk(&pk_hex)
        .await
        .map_err(|e| {
            tracing::error!("Failed to read sentinel ID pk from database: {:?}", e);
            AppError::Anyhow(e)
        })?;

    let mut headers = HeaderMap::new();
    add_cache_control_header(&mut headers, PUBLIC_KEYS_TTL);

    Ok((headers, Json(pk_with_epoch)))
}
