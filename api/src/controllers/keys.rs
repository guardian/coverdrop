use axum::{extract::Path, Extension, Json};
use chrono::{DateTime, Duration, Utc};
use common::{
    api::{
        forms::{
            DeleteJournalistForm, PatchJournalistForm, PostAdminPublicKeyForm,
            PostCoverNodeIdPublicKeyBody, PostCoverNodeIdPublicKeyForm,
            PostCoverNodeMessagingPublicKeyForm, PostCoverNodeProvisioningPublicKeyForm,
            PostJournalistForm, PostJournalistIdPublicKeyBody, PostJournalistIdPublicKeyForm,
            PostJournalistMessagingPublicKeyForm, PostJournalistProvisioningPublicKeyForm,
            RotateJournalistIdPublicKeyFormBody, RotateJournalistIdPublicKeyFormForm,
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
        models::UntrustedJournalistIdPublicKeyWithEpoch,
    },
    protocol::{
        constants::{
            COVERNODE_ID_KEY_ROTATE_AFTER_SECONDS, COVERNODE_MSG_KEY_ROTATE_AFTER_SECONDS,
            COVERNODE_PROVISIONING_KEY_ROTATE_AFTER_SECONDS,
            JOURNALIST_ID_KEY_ROTATE_AFTER_SECONDS, JOURNALIST_MSG_KEY_ROTATE_AFTER_SECONDS,
            JOURNALIST_PROVISIONING_KEY_ROTATE_AFTER_SECONDS,
        },
        keys::{
            verify_covernode_id_pk, verify_covernode_messaging_pk,
            verify_covernode_provisioning_pk, verify_journalist_id_pk,
            verify_journalist_messaging_pk, verify_journalist_provisioning_pk,
        },
        roles::{CoverNodeId, CoverNodeMessaging, JournalistId, JournalistMessaging},
    },
    system::keys::verify_admin_pk,
    time,
};
use http::HeaderMap;

use crate::{
    anchor_org_pk_cache::AnchorOrganizationPublicKeyCache,
    cache_control::{
        add_cache_control_header, PUBLIC_KEYS_TTL_IN_SECONDS, ROTATION_FORM_TTL_IN_SECONDS,
    },
    constants::MAX_NON_DESK_JOURNALIST_DESCRIPTION_LEN,
    error::AppError,
    services::database::Database,
};

pub async fn get_public_keys(
    Extension(anchor_org_pks): Extension<AnchorOrganizationPublicKeyCache>,
    Extension(db): Extension<Database>,
    Extension(default_journalist_id): Extension<Option<JournalistIdentity>>,
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

    let default_journalist_id = default_journalist_id.filter(|default_journalist_id| {
        keys.journalist_id_iter()
            .any(|existing_journalist_id| existing_journalist_id == default_journalist_id)
    });

    let keys = keys.to_untrusted();

    let mut headers = HeaderMap::new();
    add_cache_control_header(&mut headers, Duration::seconds(PUBLIC_KEYS_TTL_IN_SECONDS));

    Ok((
        headers,
        Json(UntrustedKeysAndJournalistProfiles::new(
            journalist_profiles,
            default_journalist_id,
            keys,
            max_epoch,
        )),
    ))
}

pub async fn post_journalist(
    Extension(anchor_org_pks): Extension<AnchorOrganizationPublicKeyCache>,
    Extension(db): Extension<Database>,
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

pub async fn patch_journalist(
    Extension(anchor_org_pks): Extension<AnchorOrganizationPublicKeyCache>,
    Extension(db): Extension<Database>,
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
    Extension(anchor_org_pks): Extension<AnchorOrganizationPublicKeyCache>,
    Extension(db): Extension<Database>,
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
    min_rotate_after_seconds: i64,
) -> Result<(), AppError> {
    if let Some(latest_added_at) = latest_pk_added_at {
        let seconds_since_last_rotation = latest_added_at
            .signed_duration_since(time::now())
            .num_seconds()
            .abs();

        if seconds_since_last_rotation < min_rotate_after_seconds {
            return Err(AppError::KeyRotationTooRecent);
        }
    }

    Ok(())
}

/// Function used by key upload controllers to warn if a key has been rotated too recently
/// This will be useful if the system gets stuck in a state where it is consistently rotating
fn warn_if_key_rotation_too_recent<R: Role>(
    latest_pk_added_at: Option<DateTime<Utc>>,
    min_rotate_after_seconds: i64,
) {
    if let Some(latest_added_at) = latest_pk_added_at {
        let seconds_since_last_rotation = latest_added_at
            .signed_duration_since(time::now())
            .num_seconds()
            .abs();

        if seconds_since_last_rotation < min_rotate_after_seconds {
            tracing::warn!("Key {} has been rotated too recently", R::display());
        }
    }
}

pub async fn post_covernode_provisioning_key(
    Extension(anchor_org_pks): Extension<AnchorOrganizationPublicKeyCache>,
    Extension(db): Extension<Database>,
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

    check_if_key_rotation_too_recent(
        latest_pk_added_at,
        COVERNODE_PROVISIONING_KEY_ROTATE_AFTER_SECONDS,
    )?;

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
    Extension(anchor_org_pks): Extension<AnchorOrganizationPublicKeyCache>,
    Extension(db): Extension<Database>,
    Json(form): Json<PostCoverNodeIdPublicKeyForm>,
) -> Result<Json<Epoch>, AppError> {
    let (keys, _max_epoch) = db
        .hierarchy_queries
        .key_hierarchy(&anchor_org_pks.get().await, time::now())
        .await?;

    let verifying_provisioning_pk = keys
        .find_covernode_provisioning_pk_from_raw_ed25519_pk(form.signing_pk())
        .ok_or(AppError::SigningKeyNotFound)?;

    let PostCoverNodeIdPublicKeyBody {
        covernode_id,
        covernode_id_pk,
    } = form
        .to_verified_form_data(verifying_provisioning_pk, time::now())
        .map_err(|e| {
            tracing::error!("Failed to verify form {}", e);
            AppError::SignatureVerificationFailed
        })?;

    let new_id_pk =
        verify_covernode_id_pk(&covernode_id_pk, verifying_provisioning_pk, time::now()).map_err(
            |e| {
                tracing::error!("Failed to verify covernode id key {}", e);
                AppError::SignatureVerificationFailed
            },
        )?;

    let latest_pk_added_at = db
        .covernode_key_queries
        .latest_id_pk_added_at(&covernode_id)
        .await?;

    warn_if_key_rotation_too_recent::<CoverNodeId>(
        latest_pk_added_at,
        COVERNODE_ID_KEY_ROTATE_AFTER_SECONDS,
    );

    let epoch = db
        .covernode_key_queries
        .insert_covernode_id_pk(
            &covernode_id,
            &new_id_pk,
            verifying_provisioning_pk,
            time::now(),
        )
        .await?;

    Ok(Json(epoch))
}

pub async fn post_covernode_msg_key(
    Extension(anchor_org_pks): Extension<AnchorOrganizationPublicKeyCache>,
    Extension(db): Extension<Database>,
    Json(form): Json<PostCoverNodeMessagingPublicKeyForm>,
) -> Result<Json<Epoch>, AppError> {
    let (keys, _max_epoch) = db
        .hierarchy_queries
        .key_hierarchy(&anchor_org_pks.get().await, time::now())
        .await?;

    let (covernode_id, verifying_id_pk) = keys
        .find_covernode_id_pk_from_raw_ed25519_pk(form.signing_pk())
        .ok_or(AppError::SigningKeyNotFound)?;

    let covernode_msg_pk = form
        .to_verified_form_data(verifying_id_pk, time::now())
        .map_err(|e| {
            tracing::error!("Failed to verify form {}", e);
            AppError::SignatureVerificationFailed
        })?;

    let new_msg_pk = verify_covernode_messaging_pk(&covernode_msg_pk, verifying_id_pk, time::now())
        .map_err(|e| {
            tracing::error!("Failed to verify covernode msg key {}", e);
            AppError::SignatureVerificationFailed
        })?;

    let latest_pk_added_at = db
        .covernode_key_queries
        .latest_msg_pk_added_at(covernode_id)
        .await?;

    warn_if_key_rotation_too_recent::<CoverNodeMessaging>(
        latest_pk_added_at,
        COVERNODE_MSG_KEY_ROTATE_AFTER_SECONDS,
    );

    let epoch = db
        .covernode_key_queries
        .insert_covernode_msg_pk(covernode_id, &new_msg_pk, verifying_id_pk, time::now())
        .await?;

    Ok(Json(epoch))
}

pub async fn post_journalist_provisioning_key(
    Extension(anchor_org_pks): Extension<AnchorOrganizationPublicKeyCache>,
    Extension(db): Extension<Database>,
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

    check_if_key_rotation_too_recent(
        latest_pk_added_at,
        JOURNALIST_PROVISIONING_KEY_ROTATE_AFTER_SECONDS,
    )?;

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
    Extension(anchor_org_pks): Extension<AnchorOrganizationPublicKeyCache>,
    Extension(db): Extension<Database>,
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
    Extension(db): Extension<Database>,
) -> Result<(HeaderMap, Json<Vec<JournalistIdAndPublicKeyRotationForm>>), AppError> {
    let result = db
        .journalist_queries
        .select_journalist_id_pk_rotation_forms()
        .await
        .map_err(|e| {
            tracing::error!("Failed to select journalist ID pk rotation froms: {:?}", e);
            AppError::Anyhow(e)
        })?;

    let mut headers = HeaderMap::new();
    add_cache_control_header(
        &mut headers,
        Duration::seconds(ROTATION_FORM_TTL_IN_SECONDS),
    );

    Ok((headers, Json(result)))
}

/// Upload a new journalist ID key that has been signed using a journalist provisioning key by
/// the on-premises identity services.
pub async fn post_journalist_id_key(
    Extension(anchor_org_pks): Extension<AnchorOrganizationPublicKeyCache>,
    Extension(db): Extension<Database>,
    Json(form): Json<PostJournalistIdPublicKeyForm>,
) -> Result<Json<Epoch>, AppError> {
    let (keys, _max_epoch) = db
        .hierarchy_queries
        .key_hierarchy(&anchor_org_pks.get().await, time::now())
        .await?;

    let verifying_provisioning_pk = keys
        .find_journalist_provisioning_pk_from_raw_ed25519_pk(form.signing_pk())
        .ok_or(AppError::SigningKeyNotFound)?;

    let PostJournalistIdPublicKeyBody {
        journalist_id,
        journalist_id_pk,
        from_queue,
    } = form
        .to_verified_form_data(verifying_provisioning_pk, time::now())
        .map_err(|e| {
            tracing::error!("Failed to verify form {}", e);
            AppError::SignatureVerificationFailed
        })?;

    let new_id_pk =
        verify_journalist_id_pk(&journalist_id_pk, verifying_provisioning_pk, time::now())
            .map_err(|e| {
                tracing::error!("Failed to verify journalist id key {}", e);
                AppError::SignatureVerificationFailed
            })?;

    let latest_pk_added_at = db
        .journalist_queries
        .latest_id_pk_added_at(&journalist_id)
        .await?;

    warn_if_key_rotation_too_recent::<JournalistId>(
        latest_pk_added_at,
        JOURNALIST_ID_KEY_ROTATE_AFTER_SECONDS,
    );

    let epoch = db
        .journalist_queries
        .insert_journalist_id_pk(
            &journalist_id,
            &new_id_pk,
            from_queue,
            verifying_provisioning_pk,
            time::now(),
        )
        .await?;

    Ok(Json(epoch))
}

pub async fn get_journalist_id_pk_with_epoch(
    Extension(db): Extension<Database>,
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
    add_cache_control_header(&mut headers, Duration::seconds(PUBLIC_KEYS_TTL_IN_SECONDS));

    Ok((headers, Json(pk_with_epoch)))
}

pub async fn post_journalist_msg_key(
    Extension(anchor_org_pks): Extension<AnchorOrganizationPublicKeyCache>,
    Extension(db): Extension<Database>,
    Json(form): Json<PostJournalistMessagingPublicKeyForm>,
) -> Result<Json<Epoch>, AppError> {
    let (keys, _max_epoch) = db
        .hierarchy_queries
        .key_hierarchy(&anchor_org_pks.get().await, time::now())
        .await?;

    let (journalist_id, journalist_id_pk) = keys
        .find_journalist_id_pk_from_raw_ed25519_pk(form.signing_pk())
        .ok_or(AppError::SigningKeyNotFound)?;

    // Check the form is valid
    let new_msg_pk = form
        .to_verified_form_data(journalist_id_pk, time::now())
        .map_err(|e| {
            tracing::error!("Failed to verify form {}", e);
            AppError::SignatureVerificationFailed
        })?;

    // Check the key is valid
    let new_msg_pk = verify_journalist_messaging_pk(&new_msg_pk, journalist_id_pk, time::now())
        .map_err(|e| {
            tracing::error!("Failed to verify journalist msg key {}", e);
            AppError::SignatureVerificationFailed
        })?;

    let latest_pk_added_at = db
        .journalist_queries
        .latest_msg_pk_added_at(journalist_id)
        .await?;

    warn_if_key_rotation_too_recent::<JournalistMessaging>(
        latest_pk_added_at,
        JOURNALIST_MSG_KEY_ROTATE_AFTER_SECONDS,
    );

    let epoch = db
        .journalist_queries
        .insert_journalist_msg_pk(journalist_id, new_msg_pk, journalist_id_pk, time::now())
        .await?;

    Ok(Json(epoch))
}

pub async fn post_admin_key(
    Extension(anchor_org_pks): Extension<AnchorOrganizationPublicKeyCache>,
    Extension(db): Extension<Database>,
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
