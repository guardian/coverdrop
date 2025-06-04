use crate::api::models::journalist_id::JournalistIdentity;
use crate::api::models::messages::user_to_covernode_message::{
    EncryptedUserToCoverNodeMessage, UserToCoverNodeMessage,
};
use crate::api::models::messages::user_to_journalist_message::UserToJournalistMessage;
use crate::Error;

use crate::api::models::messages::journalist_to_user_message::{
    EncryptedJournalistToUserMessage, JournalistToUserMessage,
};

use crate::crypto::TwoPartyBox;
use crate::protocol::recipient_tag::RecipientTag;
use crate::{crypto::AnonymousBox, protocol::constants::*, FixedSizeMessageText};

use super::covernode::covernode_msg_pks_from_hierarchy;
use super::keys::{CoverDropPublicKeyHierarchy, UserKeyPair, UserPublicKey};

pub fn encrypt_real_message_from_user_to_journalist_via_covernode(
    keys: &CoverDropPublicKeyHierarchy,
    user_pk: &UserPublicKey,
    journalist_id: &JournalistIdentity,
    message: FixedSizeMessageText,
) -> anyhow::Result<EncryptedUserToCoverNodeMessage> {
    let Some(journalist_msg_pk) = keys.latest_journalist_msg_pk(journalist_id) else {
        Err(Error::JournalistMessagingKeyNotFound(journalist_id.clone()))?
    };

    let user_to_journalist_message = UserToJournalistMessage::new(message, user_pk);

    let encrypted_user_to_journalist_message =
        AnonymousBox::encrypt(journalist_msg_pk, user_to_journalist_message.serialize())?;

    assert_eq!(
        encrypted_user_to_journalist_message.len(),
        USER_TO_JOURNALIST_ENCRYPTED_MESSAGE_LEN
    );

    let recipient_tag = RecipientTag::from_journalist_id(journalist_id);

    let user_to_covernode_message = UserToCoverNodeMessage::new_real_message(
        recipient_tag,
        encrypted_user_to_journalist_message,
    );

    encrypt_from_user_for_covernode(keys, user_to_covernode_message)
}

pub fn new_encrypted_cover_message_from_user_via_covernode(
    keys: &CoverDropPublicKeyHierarchy,
) -> anyhow::Result<EncryptedUserToCoverNodeMessage> {
    let user_to_covernode_message = UserToCoverNodeMessage::new_cover_message();

    encrypt_from_user_for_covernode(keys, user_to_covernode_message)
}

fn encrypt_from_user_for_covernode(
    keys: &CoverDropPublicKeyHierarchy,
    user_to_covernode_message: UserToCoverNodeMessage,
) -> anyhow::Result<EncryptedUserToCoverNodeMessage> {
    let covernode_msg_pks = covernode_msg_pks_from_hierarchy(keys)?;

    Ok(EncryptedUserToCoverNodeMessage::encrypt(
        covernode_msg_pks,
        user_to_covernode_message.serialize(),
    )?)
}

pub fn get_decrypted_user_dead_drop_message(
    user_key_pair: &UserKeyPair,
    keys: &CoverDropPublicKeyHierarchy,
    encrypted_journalist_to_user_message: &EncryptedJournalistToUserMessage,
) -> anyhow::Result<Option<(JournalistIdentity, JournalistToUserMessage)>> {
    for (candidate_id, candidate_key) in keys.journalist_msg_pk_iter() {
        let maybe_decrypted = TwoPartyBox::decrypt(
            candidate_key,
            user_key_pair.secret_key(),
            encrypted_journalist_to_user_message,
        );

        if let Ok(decrypted_serialized) = maybe_decrypted {
            return Ok(Some((
                candidate_id.clone(),
                decrypted_serialized.to_message()?,
            )));
        }
    }

    Ok(None)
}
