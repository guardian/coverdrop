use crate::api::models::dead_drops::DeadDropId;
use crate::api::models::messages::covernode_to_journalist_message::EncryptedCoverNodeToJournalistMessage;
use crate::api::models::messages::journalist_to_covernode_message::{
    EncryptedJournalistToCoverNodeMessage, JournalistToCoverNodeMessage,
};
use crate::api::models::messages::journalist_to_user_message::{
    EncryptedJournalistToUserMessage, JournalistToUserMessage,
};
use crate::api::models::messages::user_to_journalist_message::EncryptedUserToJournalistMessage;
use crate::api::models::messages::user_to_journalist_message_with_dead_drop_id::UserToJournalistMessageWithDeadDropId;
use crate::protocol::constants::JOURNALIST_TO_USER_ENCRYPTED_MESSAGE_LEN;
use crate::FixedSizeMessageText;

use super::covernode::covernode_msg_pks_from_hierarchy;
use super::keys::{
    CoverDropPublicKeyHierarchy, CoverNodeMessagingPublicKey, JournalistMessagingKeyPair,
    UserPublicKey,
};

pub fn encrypt_real_message_from_journalist_to_user_via_covernode(
    keys: &CoverDropPublicKeyHierarchy,
    user_pk: &UserPublicKey,
    journalist_key_pair: &JournalistMessagingKeyPair,
    message: &FixedSizeMessageText,
) -> anyhow::Result<EncryptedJournalistToCoverNodeMessage> {
    let journalist_to_user_message = JournalistToUserMessage::new_with_message(message.clone());

    let encrypted_journalist_to_user_message = EncryptedJournalistToUserMessage::encrypt(
        user_pk,
        journalist_key_pair.secret_key(),
        journalist_to_user_message.serialize(),
    )?;

    assert_eq!(
        encrypted_journalist_to_user_message.len(),
        JOURNALIST_TO_USER_ENCRYPTED_MESSAGE_LEN
    );

    let journalist_to_covernode_message =
        JournalistToCoverNodeMessage::new_real_message(encrypted_journalist_to_user_message);
    encrypt_from_journalist_for_covernode(keys, journalist_to_covernode_message)
}

pub fn new_encrypted_cover_message_from_journalist_via_covernode(
    keys: &CoverDropPublicKeyHierarchy,
) -> anyhow::Result<EncryptedJournalistToCoverNodeMessage> {
    let journalist_to_covernode_message = JournalistToCoverNodeMessage::new_cover_message();
    encrypt_from_journalist_for_covernode(keys, journalist_to_covernode_message)
}

fn encrypt_from_journalist_for_covernode(
    keys: &CoverDropPublicKeyHierarchy,
    journalist_to_covernode_message: JournalistToCoverNodeMessage,
) -> anyhow::Result<EncryptedJournalistToCoverNodeMessage> {
    let covernode_msg_pks = covernode_msg_pks_from_hierarchy(keys)?;

    Ok(EncryptedJournalistToCoverNodeMessage::encrypt(
        covernode_msg_pks,
        journalist_to_covernode_message.serialize(),
    )?)
}

/// Attempt to decrypt a given dead drop message for a journalist
/// Should be passed all available CoverNode messaging public keys and the
/// messaging key pairs for a single journalist.
pub fn get_decrypted_journalist_dead_drop_message(
    covernode_msg_pks: &[&CoverNodeMessagingPublicKey],
    journalist_msg_key_pairs: &[JournalistMessagingKeyPair],
    encrypted_user_to_journalist_message: &EncryptedCoverNodeToJournalistMessage,
    dead_drop_id: DeadDropId,
) -> Option<UserToJournalistMessageWithDeadDropId> {
    let mut maybe_outer_decrypted = None;

    'outer_message_loop: for covernode_msg_pk in covernode_msg_pks.iter() {
        for journalist_msg_key_pair in journalist_msg_key_pairs {
            // Attempt to decrypt the outer C2J layer
            let maybe_outer_decrypted_serialized = EncryptedCoverNodeToJournalistMessage::decrypt(
                *covernode_msg_pk,
                journalist_msg_key_pair.secret_key(),
                encrypted_user_to_journalist_message,
            );

            if let Ok(outer_decrypted_serialized) = maybe_outer_decrypted_serialized {
                // We found our key pairs
                maybe_outer_decrypted = Some(outer_decrypted_serialized.to_message());
                break 'outer_message_loop;
            }
        }
    }

    if let Some(outer_decrypted) = maybe_outer_decrypted {
        // Must iterate over all the keys again since we U2J message might have been sent
        // before a key rotation, so it might not be encrypted with the most recent messaging key
        for journalist_msg_key_pair in journalist_msg_key_pairs {
            let inner_maybe_decrypted = EncryptedUserToJournalistMessage::decrypt(
                journalist_msg_key_pair,
                &outer_decrypted.payload,
            );

            if let Ok(inner_decrypted_serialized) = inner_maybe_decrypted {
                return Some(UserToJournalistMessageWithDeadDropId {
                    u2j_message: inner_decrypted_serialized.to_message(),
                    dead_drop_id,
                });
            }
        }
    }

    // Could not decrypt message - it was not for this journalist
    None
}

#[cfg(test)]
mod test {
    use crate::{
        api::models::messages::{
            covernode_to_journalist_message::{
                CoverNodeToJournalistMessage, EncryptedCoverNodeToJournalistMessage,
            },
            user_to_journalist_message::UserToJournalistMessage,
        },
        crypto::{keys::encryption::UnsignedEncryptionKeyPair, AnonymousBox},
        protocol::{
            keys::{
                generate_covernode_messaging_key_pair, generate_journalist_messaging_key_pair,
                test::{generate_protocol_keys, ProtocolKeys},
            },
            roles::User,
        },
        time, FixedSizeMessageText,
    };

    use super::get_decrypted_journalist_dead_drop_message;

    #[test]
    fn c2j_and_u2j_messages_decrypt_correctly_when_different_journalist_msg_pk_is_used() {
        let now = time::now();
        let ProtocolKeys {
            journalist_id_key_pair,
            covernode_id_key_pair,
            ..
        } = generate_protocol_keys(now);

        let user_key_pair = UnsignedEncryptionKeyPair::<User>::generate();

        let journalist_msg_key_pair_1 =
            generate_journalist_messaging_key_pair(&journalist_id_key_pair, now);
        let covernode_msg_key_pair =
            generate_covernode_messaging_key_pair(&covernode_id_key_pair, now);

        // Encrypt U2J
        let user_to_journalist_message = UserToJournalistMessage::new(
            FixedSizeMessageText::new("text").expect("Create fixed size message text"),
            user_key_pair.public_key(),
        );
        let user_to_journalist_message = user_to_journalist_message.serialize();
        let user_to_journalist_message = AnonymousBox::encrypt(
            journalist_msg_key_pair_1.public_key(),
            user_to_journalist_message,
        )
        .expect("Encrypt U2J message");

        // Journlist rotates msg key
        let journalist_msg_key_pair_2 =
            generate_journalist_messaging_key_pair(&journalist_id_key_pair, now);

        // Encrypt C2J
        let covernode_to_journalist_message =
            CoverNodeToJournalistMessage::new(user_to_journalist_message.clone());
        let covernode_to_journalist_message = covernode_to_journalist_message.serialize();
        let covernode_to_journalist_message = EncryptedCoverNodeToJournalistMessage::encrypt(
            journalist_msg_key_pair_2.public_key(), // Note different public key
            covernode_msg_key_pair.secret_key(),
            covernode_to_journalist_message,
        )
        .expect("Encrypt CoverNode to Journalist message");

        // Attempt decrypt
        get_decrypted_journalist_dead_drop_message(
            &[covernode_msg_key_pair.public_key()],
            &[journalist_msg_key_pair_1, journalist_msg_key_pair_2],
            &covernode_to_journalist_message,
            0,
        )
        .expect("Decrypt message");
    }
}
