use crate::api::models::dead_drops::DeadDropId;
use crate::api::models::messages::covernode_to_journalist_message::EncryptedCoverNodeToJournalistMessage;
use crate::api::models::messages::decrypted_user_to_journalist_message_with_metadata::{
    DecryptedUserToJournalistMessageWithMetadata, UserToJournalistMessagePayload,
};
use crate::api::models::messages::journalist_to_covernode_message::{
    EncryptedJournalistToCoverNodeMessage, JournalistToCoverNodeMessage,
};
use crate::api::models::messages::journalist_to_user_message::{
    EncryptedJournalistToUserMessage, JournalistToUserMessage,
};
use crate::api::models::messages::user_to_journalist_message::{
    EncryptedUserToJournalistMessage, EncryptedUserToJournalistPayload, UserToJournalistMessage,
    UserToJournalistPayload,
};
use crate::protocol::constants::JOURNALIST_TO_USER_ENCRYPTED_MESSAGE_LEN;
use crate::FixedSizeMessageText;
use chrono::{DateTime, Utc};

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
    dead_drop_created_at: DateTime<Utc>,
) -> Option<DecryptedUserToJournalistMessageWithMetadata> {
    // Attempt to decrypt the outer C2J layer. If no combination of keys works then the
    // message was not for this journalist.
    let outer_decrypted = covernode_msg_pks
        .iter()
        .find_map(|covernode_msg_pk| {
            journalist_msg_key_pairs
                .iter()
                .find_map(|journalist_msg_key_pair| {
                    EncryptedCoverNodeToJournalistMessage::decrypt(
                        *covernode_msg_pk,
                        journalist_msg_key_pair.secret_key(),
                        encrypted_user_to_journalist_message,
                    )
                    .ok()
                })
        })?
        .to_message();

    // Must iterate over all the keys again since the U2J message might have been sent
    // before a key rotation, so it might not be encrypted with the most recent messaging key
    let (reply_key, payload) =
        journalist_msg_key_pairs
            .iter()
            .find_map(|journalist_msg_key_pair| {
                let decrypted_message = EncryptedUserToJournalistMessage::decrypt(
                    journalist_msg_key_pair,
                    &outer_decrypted.payload,
                )
                .ok()?
                .to_message();
                try_decrypt_u2j_message(journalist_msg_key_pair, &decrypted_message)
            })?;

    Some(DecryptedUserToJournalistMessageWithMetadata {
        reply_key,
        payload,
        unsigned_dead_drop_id: dead_drop_id,
        dead_drop_created_at,
    })
}

/// During migration to authenticated U2J messages we need to support both types i.e.
/// [UserToJournalistPayload::Authenticated] and [serToJournalistPayload::LegacyUnauthenticated].
/// This function takes a U2J messages and normalises the payload to a [UserToJournalistMessagePayload].
/// This normalisation allows both message types to co-exist during migration but will be removed
/// once authenticated messages are fully rolled-out.
pub fn try_decrypt_u2j_message(
    journalist_msg_key_pair: &JournalistMessagingKeyPair,
    user_to_journalist_message: &UserToJournalistMessage,
) -> Option<(UserPublicKey, UserToJournalistMessagePayload)> {
    let reply_key = &user_to_journalist_message.reply_key;

    match &user_to_journalist_message.payload {
        UserToJournalistPayload::Authenticated(encrypted_payload) => {
            EncryptedUserToJournalistPayload::decrypt(
                reply_key,
                journalist_msg_key_pair.secret_key(),
                encrypted_payload,
            )
            .map(|fixed_size_message_text| {
                (
                    reply_key.clone(),
                    UserToJournalistMessagePayload::Authenticated(fixed_size_message_text),
                )
            })
            .ok()
        }
        UserToJournalistPayload::LegacyUnauthenticated(message) => Some((
            reply_key.clone(),
            UserToJournalistMessagePayload::LegacyUnauthenticated(message.clone()),
        )),
    }
}

#[cfg(test)]
mod test {
    use crate::{
        api::models::messages::{
            covernode_to_journalist_message::{
                CoverNodeToJournalistMessage, EncryptedCoverNodeToJournalistMessage,
            },
            user_to_journalist_message::{
                EncryptedUserToJournalistPayload, UserToJournalistMessage,
            },
        },
        crypto::{keys::encryption::UnsignedEncryptionKeyPair, AnonymousBox},
        protocol::{
            keys::{
                generate_covernode_messaging_key_pair, generate_journalist_messaging_key_pair,
                test::{generate_protocol_keys, ProtocolKeys},
            },
            roles::User,
        },
        time, FixedSizeAuthenticatedMessageText,
    };

    use super::{get_decrypted_journalist_dead_drop_message, try_decrypt_u2j_message};
    use crate::api::models::messages::decrypted_user_to_journalist_message_with_metadata::UserToJournalistMessagePayload;
    use crate::api::models::messages::user_to_journalist_message::UserToJournalistPayload;
    use crate::crypto::keys::encryption::traits::PublicEncryptionKey as _;
    use crate::crypto::TwoPartyBox;
    use crate::protocol::keys::{CoverNodeMessagingKeyPair, JournalistMessagingKeyPair};
    use crate::FixedSizeMessageText;
    use chrono::Utc;

    /// Build an authenticated U2J message from `user_key_pair` addressed to `journalist_msg_key_pair`.
    fn new_authenticated_u2j_message(
        journalist_msg_key_pair: &JournalistMessagingKeyPair,
        user_key_pair: &UnsignedEncryptionKeyPair<User>,
        message_text: &str,
    ) -> UserToJournalistMessage {
        let message = FixedSizeAuthenticatedMessageText::new(message_text)
            .expect("Create fixed size authenticated message text");

        UserToJournalistMessage::new_authenticated(
            user_key_pair,
            journalist_msg_key_pair.public_key(),
            message,
        )
    }

    /// Build a dead drop message as the CoverNode would. The U2J and innermost layers are
    /// encrypted to `u2j_msg_key_pair`, which may differ from the key the CoverNode wraps with.
    fn new_c2j_dead_drop_message(
        covernode_msg_key_pair: &CoverNodeMessagingKeyPair,
        c2j_msg_key_pair: &JournalistMessagingKeyPair,
        u2j_msg_key_pair: &JournalistMessagingKeyPair,
        user_key_pair: &UnsignedEncryptionKeyPair<User>,
        message_text: &str,
    ) -> EncryptedCoverNodeToJournalistMessage {
        let u2j_message =
            new_authenticated_u2j_message(u2j_msg_key_pair, user_key_pair, message_text);

        let encrypted_u2j_message =
            AnonymousBox::encrypt(u2j_msg_key_pair.public_key(), u2j_message.serialize())
                .expect("Encrypt U2J message");

        EncryptedCoverNodeToJournalistMessage::encrypt(
            c2j_msg_key_pair.public_key(),
            covernode_msg_key_pair.secret_key(),
            CoverNodeToJournalistMessage::new(encrypted_u2j_message).serialize(),
        )
        .expect("Encrypt C2J message")
    }

    // Test unpack_inner_u2j_message functionality

    #[test]
    fn authenticated_message_unpacks_with_matching_key_pair() {
        let now = time::now();
        let ProtocolKeys {
            journalist_id_key_pair,
            ..
        } = generate_protocol_keys(now);

        let journalist_msg_key_pair =
            generate_journalist_messaging_key_pair(&journalist_id_key_pair, now);
        let user_key_pair = UnsignedEncryptionKeyPair::<User>::generate();

        let u2j_message =
            new_authenticated_u2j_message(&journalist_msg_key_pair, &user_key_pair, "hello");

        let (reply_key, payload) = try_decrypt_u2j_message(&journalist_msg_key_pair, &u2j_message)
            .expect("Unpack authenticated message");

        assert_eq!(payload.to_string().unwrap(), "hello");
        assert_eq!(reply_key.as_bytes(), user_key_pair.public_key().as_bytes());
    }

    #[test]
    fn authenticated_message_round_trips_an_empty_message() {
        let now = time::now();
        let ProtocolKeys {
            journalist_id_key_pair,
            ..
        } = generate_protocol_keys(now);

        let journalist_msg_key_pair =
            generate_journalist_messaging_key_pair(&journalist_id_key_pair, now);
        let user_key_pair = UnsignedEncryptionKeyPair::<User>::generate();

        let u2j_message =
            new_authenticated_u2j_message(&journalist_msg_key_pair, &user_key_pair, "");

        let (reply_key, payload) = try_decrypt_u2j_message(&journalist_msg_key_pair, &u2j_message)
            .expect("Unpack authenticated message");

        assert_eq!(payload.to_string().unwrap(), "");
        assert_eq!(reply_key.as_bytes(), user_key_pair.public_key().as_bytes());
    }

    #[test]
    fn authenticated_message_round_trips_multi_byte_characters() {
        let now = time::now();
        let ProtocolKeys {
            journalist_id_key_pair,
            ..
        } = generate_protocol_keys(now);

        let journalist_msg_key_pair =
            generate_journalist_messaging_key_pair(&journalist_id_key_pair, now);
        let user_key_pair = UnsignedEncryptionKeyPair::<User>::generate();

        let message_text = "héllo 👋 世界";
        let u2j_message =
            new_authenticated_u2j_message(&journalist_msg_key_pair, &user_key_pair, message_text);

        let (reply_key, payload) = try_decrypt_u2j_message(&journalist_msg_key_pair, &u2j_message)
            .expect("Unpack authenticated message");

        assert_eq!(payload.to_string().unwrap(), message_text);
        assert_eq!(reply_key.as_bytes(), user_key_pair.public_key().as_bytes());
    }

    #[test]
    fn authenticated_message_returns_none_for_a_non_matching_key_pair() {
        let now = time::now();
        let ProtocolKeys {
            journalist_id_key_pair,
            ..
        } = generate_protocol_keys(now);

        let recipient_msg_key_pair =
            generate_journalist_messaging_key_pair(&journalist_id_key_pair, now);
        let other_msg_key_pair =
            generate_journalist_messaging_key_pair(&journalist_id_key_pair, now);
        let user_key_pair = UnsignedEncryptionKeyPair::<User>::generate();

        let u2j_message =
            new_authenticated_u2j_message(&recipient_msg_key_pair, &user_key_pair, "hello");

        assert!(try_decrypt_u2j_message(&other_msg_key_pair, &u2j_message).is_none());
    }

    #[test]
    fn authenticated_message_returns_none_when_reply_key_does_not_match_sender() {
        let now = time::now();
        let ProtocolKeys {
            journalist_id_key_pair,
            ..
        } = generate_protocol_keys(now);

        let journalist_msg_key_pair =
            generate_journalist_messaging_key_pair(&journalist_id_key_pair, now);
        let user_key_pair = UnsignedEncryptionKeyPair::<User>::generate();
        let attacker_key_pair = UnsignedEncryptionKeyPair::<User>::generate();

        let u2j_message =
            new_authenticated_u2j_message(&journalist_msg_key_pair, &user_key_pair, "hello");

        // Swapping the reply key must invalidate the inner message's authentication tag
        let tampered_message = UserToJournalistMessage {
            reply_key: attacker_key_pair.public_key().clone(),
            payload: u2j_message.payload,
        };

        assert!(try_decrypt_u2j_message(&journalist_msg_key_pair, &tampered_message).is_none());
    }

    #[test]
    fn authenticated_message_returns_none_when_ciphertext_is_tampered_with() {
        let now = time::now();
        let ProtocolKeys {
            journalist_id_key_pair,
            ..
        } = generate_protocol_keys(now);

        let journalist_msg_key_pair =
            generate_journalist_messaging_key_pair(&journalist_id_key_pair, now);
        let user_key_pair = UnsignedEncryptionKeyPair::<User>::generate();

        let u2j_message =
            new_authenticated_u2j_message(&journalist_msg_key_pair, &user_key_pair, "hello");

        let UserToJournalistPayload::Authenticated(inner_message) = u2j_message.payload else {
            panic!("Expected an authenticated message");
        };
        let mut bytes = inner_message.as_bytes().to_vec();
        let last = bytes.len() - 1;
        bytes[last] ^= 0xff;

        let tampered_message = UserToJournalistMessage {
            reply_key: user_key_pair.public_key().clone(),
            payload: UserToJournalistPayload::Authenticated(
                EncryptedUserToJournalistPayload::from_vec_unchecked(bytes),
            ),
        };

        assert!(try_decrypt_u2j_message(&journalist_msg_key_pair, &tampered_message).is_none());
    }

    #[test]
    fn plain_message_is_returned_without_a_matching_key_pair() {
        let now = time::now();
        let ProtocolKeys {
            journalist_id_key_pair,
            ..
        } = generate_protocol_keys(now);

        let journalist_msg_key_pair =
            generate_journalist_messaging_key_pair(&journalist_id_key_pair, now);
        let user_key_pair = UnsignedEncryptionKeyPair::<User>::generate();
        let message = FixedSizeMessageText::new("hello").expect("Create fixed size message text");

        #[allow(deprecated)]
        let u2j_message = UserToJournalistMessage::new(message.clone(), user_key_pair.public_key());

        let (reply_key, payload) = try_decrypt_u2j_message(&journalist_msg_key_pair, &u2j_message)
            .expect("Unpack plain message");

        assert_eq!(
            payload,
            UserToJournalistMessagePayload::LegacyUnauthenticated(message)
        );
        assert_eq!(payload.to_string().unwrap(), "hello");
        assert_eq!(reply_key.as_bytes(), user_key_pair.public_key().as_bytes());
    }

    // Test get_decrypted_journalist_dead_drop_message functionality

    #[test]
    fn dead_drop_message_decrypts_when_matching_key_pairs_are_not_first() {
        let now = time::now();
        let ProtocolKeys {
            journalist_id_key_pair,
            covernode_id_key_pair,
            ..
        } = generate_protocol_keys(now);

        let covernode_msg_key_pair =
            generate_covernode_messaging_key_pair(&covernode_id_key_pair, now);
        let user_key_pair = UnsignedEncryptionKeyPair::<User>::generate();

        let old_msg_key_pair = generate_journalist_messaging_key_pair(&journalist_id_key_pair, now);
        let new_msg_key_pair = generate_journalist_messaging_key_pair(&journalist_id_key_pair, now);
        let unused_msg_key_pair =
            generate_journalist_messaging_key_pair(&journalist_id_key_pair, now);

        // The source encrypted to the old key before the rotation was published, so the CoverNode
        // wrapped it with a different, newer key
        let dead_drop_message = new_c2j_dead_drop_message(
            &covernode_msg_key_pair,
            &new_msg_key_pair,
            &old_msg_key_pair,
            &user_key_pair,
            "hello",
        );

        let decrypted_message = get_decrypted_journalist_dead_drop_message(
            &[covernode_msg_key_pair.public_key()],
            &[unused_msg_key_pair, new_msg_key_pair, old_msg_key_pair],
            &dead_drop_message,
            7,
            now,
        )
        .expect("Decrypt message");

        assert_eq!(decrypted_message.payload.to_string().unwrap(), "hello");
        assert_eq!(
            decrypted_message.reply_key.as_bytes(),
            user_key_pair.public_key().as_bytes()
        );
        assert_eq!(decrypted_message.unsigned_dead_drop_id, 7);
        assert_eq!(decrypted_message.dead_drop_created_at, now);
    }

    #[test]
    fn dead_drop_message_decrypts_when_matching_covernode_msg_pk_is_not_first() {
        let now = time::now();
        let ProtocolKeys {
            journalist_id_key_pair,
            covernode_id_key_pair,
            ..
        } = generate_protocol_keys(now);

        let covernode_msg_key_pair =
            generate_covernode_messaging_key_pair(&covernode_id_key_pair, now);
        let rotated_covernode_msg_key_pair =
            generate_covernode_messaging_key_pair(&covernode_id_key_pair, now);
        let journalist_msg_key_pair =
            generate_journalist_messaging_key_pair(&journalist_id_key_pair, now);
        let user_key_pair = UnsignedEncryptionKeyPair::<User>::generate();

        let dead_drop_message = new_c2j_dead_drop_message(
            &covernode_msg_key_pair,
            &journalist_msg_key_pair,
            &journalist_msg_key_pair,
            &user_key_pair,
            "hello",
        );

        let decrypted_message = get_decrypted_journalist_dead_drop_message(
            &[
                rotated_covernode_msg_key_pair.public_key(),
                covernode_msg_key_pair.public_key(),
            ],
            &[journalist_msg_key_pair],
            &dead_drop_message,
            0,
            now,
        )
        .expect("Decrypt message");

        assert_eq!(decrypted_message.payload.to_string().unwrap(), "hello");
    }

    #[test]
    fn dead_drop_message_returns_none_when_the_u2j_key_pair_is_missing() {
        let now = time::now();
        let ProtocolKeys {
            journalist_id_key_pair,
            covernode_id_key_pair,
            ..
        } = generate_protocol_keys(now);

        let covernode_msg_key_pair =
            generate_covernode_messaging_key_pair(&covernode_id_key_pair, now);
        let user_key_pair = UnsignedEncryptionKeyPair::<User>::generate();

        let old_msg_key_pair = generate_journalist_messaging_key_pair(&journalist_id_key_pair, now);
        let new_msg_key_pair = generate_journalist_messaging_key_pair(&journalist_id_key_pair, now);

        let dead_drop_message = new_c2j_dead_drop_message(
            &covernode_msg_key_pair,
            &new_msg_key_pair,
            &old_msg_key_pair,
            &user_key_pair,
            "hello",
        );

        // The C2J layer opens with the new key but the U2J layer needs the old one
        assert!(get_decrypted_journalist_dead_drop_message(
            &[covernode_msg_key_pair.public_key()],
            &[new_msg_key_pair],
            &dead_drop_message,
            0,
            now,
        )
        .is_none());
    }

    #[test]
    fn dead_drop_message_returns_none_when_the_covernode_msg_pk_is_missing() {
        let now = time::now();
        let ProtocolKeys {
            journalist_id_key_pair,
            covernode_id_key_pair,
            ..
        } = generate_protocol_keys(now);

        let covernode_msg_key_pair =
            generate_covernode_messaging_key_pair(&covernode_id_key_pair, now);
        let other_covernode_msg_key_pair =
            generate_covernode_messaging_key_pair(&covernode_id_key_pair, now);
        let journalist_msg_key_pair =
            generate_journalist_messaging_key_pair(&journalist_id_key_pair, now);
        let user_key_pair = UnsignedEncryptionKeyPair::<User>::generate();

        let dead_drop_message = new_c2j_dead_drop_message(
            &covernode_msg_key_pair,
            &journalist_msg_key_pair,
            &journalist_msg_key_pair,
            &user_key_pair,
            "hello",
        );

        assert!(get_decrypted_journalist_dead_drop_message(
            &[other_covernode_msg_key_pair.public_key()],
            &[journalist_msg_key_pair],
            &dead_drop_message,
            0,
            now,
        )
        .is_none());
    }

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

        let message_text = "text";

        let message = FixedSizeAuthenticatedMessageText::new(message_text)
            .expect("Create fixed size message text");

        // Encrypt U2J
        let user_to_journalist_message = UserToJournalistMessage::new_authenticated(
            &user_key_pair,
            journalist_msg_key_pair_1.public_key(),
            message,
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
        let decrypted_message = get_decrypted_journalist_dead_drop_message(
            &[covernode_msg_key_pair.public_key()],
            &[journalist_msg_key_pair_1, journalist_msg_key_pair_2],
            &covernode_to_journalist_message,
            0,
            Utc::now(),
        )
        .expect("Decrypt message");

        assert_eq!(
            decrypted_message.payload.to_string().unwrap().as_str(),
            message_text
        );
    }
}
