use chrono::{DateTime, Utc};

use crate::api::models::dead_drops::{
    JournalistToUserDeadDrop, JournalistToUserDeadDropSignatureDataV2,
    UnpublishedJournalistToUserDeadDrop, UnpublishedUserToJournalistDeadDrop,
    UnverifiedJournalistToUserDeadDropsList, UnverifiedUserToJournalistDeadDropsList,
    UserToJournalistDeadDrop, UserToJournalistDeadDropSignatureDataV2,
};
use crate::api::models::messages::covernode_to_journalist_message::{
    CoverNodeToJournalistMessage, EncryptedCoverNodeToJournalistMessage,
};
use crate::api::models::messages::journalist_to_covernode_message::{
    EncryptedJournalistToCoverNodeMessage, JournalistToCoverNodeMessage,
};
use crate::api::models::messages::user_to_covernode_message::{
    EncryptedUserToCoverNodeMessage, UserToCoverNodeMessage,
};
use crate::api::models::messages::user_to_journalist_message::EncryptedUserToJournalistMessage;
use crate::crypto::keys::signing::traits::PublicSigningKey;
use crate::crypto::{MultiAnonymousBox, Verified};
use crate::protocol::constants::COVERNODE_TO_JOURNALIST_ENCRYPTED_MESSAGE_LEN;
use crate::Error;

use super::constants::COVERNODE_WRAPPING_KEY_COUNT;
use super::keys::{
    CoverDropPublicKeyHierarchy, CoverNodeMessagingKeyPair, CoverNodeMessagingPublicKey,
    JournalistMessagingPublicKey,
};

//
// Keys
//

/// When sending messages to the CoverNode we want to encrypt with multiple CoverNode's
/// public keys in case a node fails. This function gets a sufficient array of candiate.
pub fn covernode_msg_pks_from_hierarchy(
    keys: &CoverDropPublicKeyHierarchy,
) -> anyhow::Result<[&CoverNodeMessagingPublicKey; COVERNODE_WRAPPING_KEY_COUNT]> {
    let mut covernode_msg_pks = keys
        .latest_covernode_msg_pk_iter()
        .map(|(_, msg_pk)| msg_pk)
        .collect::<Vec<_>>();

    if covernode_msg_pks.is_empty() {
        Err(Error::CoverNodeMessagingKeyNotFound)?;
    };

    if covernode_msg_pks.len() < COVERNODE_WRAPPING_KEY_COUNT {
        tracing::warn!(
            "Not enough CoverDrop messaging keys from different nodes available, this can lead to lack of reliability"
        );
    }

    // If we don't have enough CoverNode keys, pad out the array
    while covernode_msg_pks.len() < COVERNODE_WRAPPING_KEY_COUNT {
        covernode_msg_pks.push(covernode_msg_pks[0]);
    }

    let keys = covernode_msg_pks[..COVERNODE_WRAPPING_KEY_COUNT]
        .try_into()
        .expect("Key slice should be padded to the correct length");

    Ok(keys)
}

//
// Decryption
//

pub fn decrypt_user_message(
    covernode_msg_key_pair: &CoverNodeMessagingKeyPair,
    encrypted_user_to_covernode_message: &EncryptedUserToCoverNodeMessage,
) -> anyhow::Result<UserToCoverNodeMessage> {
    let decrypted =
        MultiAnonymousBox::decrypt(covernode_msg_key_pair, encrypted_user_to_covernode_message)?;

    Ok(decrypted.to_message())
}

pub fn decrypt_journalist_message(
    covernode_msg_key_pair: &CoverNodeMessagingKeyPair,
    encrypted_journalist_to_covernode_message: &EncryptedJournalistToCoverNodeMessage,
) -> anyhow::Result<JournalistToCoverNodeMessage> {
    let decrypted_serialized = MultiAnonymousBox::decrypt(
        covernode_msg_key_pair,
        encrypted_journalist_to_covernode_message,
    )?;

    Ok(decrypted_serialized.to_message())
}

//
// Encryption
//

pub fn encrypt_message_to_journalist(
    covernode_msg_key_pair: &CoverNodeMessagingKeyPair,
    journalist_msg_pk: &JournalistMessagingPublicKey,
    encrypted_user_to_journalist_message: EncryptedUserToJournalistMessage,
) -> anyhow::Result<EncryptedCoverNodeToJournalistMessage> {
    let covernode_to_journalist_message =
        CoverNodeToJournalistMessage::new(encrypted_user_to_journalist_message);
    let encrypted_covernode_to_journalist_message = EncryptedCoverNodeToJournalistMessage::encrypt(
        journalist_msg_pk,
        covernode_msg_key_pair.secret_key(),
        covernode_to_journalist_message.serialize(),
    )?;
    assert_eq!(
        encrypted_covernode_to_journalist_message.len(),
        COVERNODE_TO_JOURNALIST_ENCRYPTED_MESSAGE_LEN
    );

    Ok(encrypted_covernode_to_journalist_message)
}

//
// Verification
//

pub fn verify_user_to_journalist_dead_drop_list(
    keys: &CoverDropPublicKeyHierarchy,
    dead_drop_list: UnverifiedUserToJournalistDeadDropsList,
    now: DateTime<Utc>,
) -> Vec<UserToJournalistDeadDrop> {
    dead_drop_list
        .dead_drops
        .into_iter()
        .filter_map(|dead_drop| {
            // For each ID PK, check the dead drop
            for (_, id_pk) in keys.covernode_id_pk_iter() {
                let signature_data = dead_drop.signature_data();

                if id_pk
                    .verify(&signature_data, &dead_drop.signature, now)
                    .is_ok()
                {
                    let verified_dead_drop = UserToJournalistDeadDrop::new(
                        dead_drop.id,
                        dead_drop.created_at,
                        dead_drop.data.deserialize(),
                        dead_drop.signature,
                        dead_drop.epoch,
                    );

                    return Some(verified_dead_drop);
                }
            }

            // TODO this is maybe more severe than a warning? It implies the client has
            // lost sync with the CoverNode ID public keys OR the dead drops are being signed
            // by the wrong key.
            tracing::warn!("Failed to verify user to journalist dead drop in dead drop list");
            None
        })
        .collect()
}

pub fn verify_journalist_to_user_dead_drop_list(
    keys: &CoverDropPublicKeyHierarchy,
    dead_drop_list: &UnverifiedJournalistToUserDeadDropsList,
    now: DateTime<Utc>,
) -> Vec<JournalistToUserDeadDrop> {
    dead_drop_list
        .dead_drops
        .iter()
        .filter_map(|dead_drop| {
            // For each ID PK, check the dead drop
            for (_, id_pk) in keys.covernode_id_pk_iter() {
                let signature_data = dead_drop.signature_data();

                if id_pk
                    .verify(&signature_data, &dead_drop.signature, now)
                    .is_ok()
                {
                    let verified_dead_drop = JournalistToUserDeadDrop::new(
                        dead_drop.id,
                        dead_drop.created_at,
                        dead_drop.data.deserialize(),
                    );

                    return Some(verified_dead_drop);
                }
            }

            tracing::warn!("Failed to verify journalist to user dead drop in dead drop list");
            None
        })
        .collect::<Vec<JournalistToUserDeadDrop>>()
}

/// Iterates through available verified covernode identity keys, trying to find one that can
/// verify the provided user to journalist dead drop
pub fn verify_unpublished_user_to_journalist_dead_drop(
    keys: &CoverDropPublicKeyHierarchy,
    dead_drop: UnpublishedUserToJournalistDeadDrop,
    now: DateTime<Utc>,
) -> anyhow::Result<Verified<UnpublishedUserToJournalistDeadDrop>> {
    let signature_data = UserToJournalistDeadDropSignatureDataV2::new(
        &dead_drop.data,
        dead_drop.created_at,
        dead_drop.epoch,
    );

    for (_, id_pk) in keys.covernode_id_pk_iter() {
        if id_pk
            .verify(&signature_data, &dead_drop.signature, now)
            .is_ok()
        {
            return Ok(Verified(dead_drop));
        }
    }

    anyhow::bail!(
        "Failed to verify user to journalist dead drop with available CoverNode identity keys"
    );
}

/// Iterates through available verified covernode identity keys, trying to find one that can
/// verify the provided user to journalist dead drop
///
/// Used when the CoverNode is submitting a new dead drop but it has yet to be assigned an ID
pub fn verify_unpublished_journalist_to_user_dead_drop(
    keys: &CoverDropPublicKeyHierarchy,
    dead_drop: UnpublishedJournalistToUserDeadDrop,
    now: DateTime<Utc>,
) -> anyhow::Result<Verified<UnpublishedJournalistToUserDeadDrop>> {
    let signature_data =
        JournalistToUserDeadDropSignatureDataV2::new(&dead_drop.data, dead_drop.created_at);

    for (_, id_pk) in keys.covernode_id_pk_iter() {
        if id_pk
            .verify(&signature_data, &dead_drop.signature, now)
            .is_ok()
        {
            return Ok(Verified(dead_drop));
        }
    }

    anyhow::bail!(
        "Failed to verify journalist to user dead drop with available CoverNode identity keys"
    );
}
