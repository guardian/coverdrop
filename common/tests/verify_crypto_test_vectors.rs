use chrono::{DateTime, Utc};
use common::api::models::dead_drops::SerializedUserToJournalistDeadDropMessages;
use common::crypto::keys::encryption::{
    PublicEncryptionKey, SecretEncryptionKey, SignedEncryptionKeyPair, UnsignedEncryptionKeyPair,
};
use common::crypto::keys::key_certificate_data::KeyCertificateData;
use common::crypto::keys::role::Test;
use common::crypto::keys::signing::{traits, PublicSigningKey, UnsignedSigningKeyPair};
use common::crypto::keys::{X25519PublicKey, X25519SecretKey};
use common::crypto::{AnonymousBox, MultiAnonymousBox, Signable, Signature, TwoPartyBox};
use common::protocol::journalist::get_decrypted_journalist_dead_drop_message;
use common::protocol::keys::{
    CoverNodeMessagingPublicKey, JournalistMessagingPublicKey, UserPublicKey,
};
use common::protocol::roles::{CoverNodeMessaging, JournalistMessaging};
use common::time;

#[test]
fn test_anonymous_box() -> anyhow::Result<()> {
    let recipient_pk = PublicEncryptionKey::<Test>::new(X25519PublicKey::from(
        include_bytes!("vectors/anonymous_box/01_recipient_pk").to_owned(),
    ));
    let recipient_sk = SecretEncryptionKey::new(X25519SecretKey::from(
        include_bytes!("vectors/anonymous_box/02_recipient_sk").to_owned(),
    ));

    let recipient_key_pair = UnsignedEncryptionKeyPair::new(recipient_pk, recipient_sk);

    let message = include_bytes!("vectors/anonymous_box/03_message").to_vec();
    let anonymous_box: AnonymousBox<Vec<u8>> = AnonymousBox::from_vec_unchecked(
        include_bytes!("vectors/anonymous_box/04_anonymous_box").to_vec(),
    );

    // ensure decryption results in original message
    let actual = AnonymousBox::decrypt(&recipient_key_pair, &anonymous_box)?;
    assert_eq!(message, actual);

    Ok(())
}

#[test]
fn test_two_party_box() -> anyhow::Result<()> {
    let sender_pk = PublicEncryptionKey::<Test>::new(X25519PublicKey::from(
        include_bytes!("vectors/two_party_box/01_sender_pk").to_owned(),
    ));

    let recipient_pk = PublicEncryptionKey::<Test>::new(X25519PublicKey::from(
        include_bytes!("vectors/anonymous_box/01_recipient_pk").to_owned(),
    ));
    let recipient_sk = SecretEncryptionKey::new(X25519SecretKey::from(
        include_bytes!("vectors/two_party_box/04_recipient_sk").to_owned(),
    ));

    let recipient_key_pair = UnsignedEncryptionKeyPair::new(recipient_pk, recipient_sk);

    let message = include_bytes!("vectors/two_party_box/05_message").to_vec();
    let anonymous_box: TwoPartyBox<Vec<u8>> = TwoPartyBox::from_vec_unchecked(
        include_bytes!("vectors/two_party_box/06_two_party_box").to_vec(),
    );

    // ensure decryption results in original message
    let actual = TwoPartyBox::decrypt(&sender_pk, recipient_key_pair.secret_key(), &anonymous_box)?;
    assert_eq!(message, actual);

    Ok(())
}

#[test]
fn test_multi_anonymous_box() -> anyhow::Result<()> {
    const NUM_RECIPIENTS: usize = 2;
    let recipient_1_pk = PublicEncryptionKey::<Test>::new(x25519_dalek::PublicKey::from(
        include_bytes!("vectors/multi_anonymous_box/01_recipient_1_pk").to_owned(),
    ));
    let recipient_1_sk = SecretEncryptionKey::new(X25519SecretKey::from(
        include_bytes!("vectors/multi_anonymous_box/02_recipient_1_sk").to_owned(),
    ));

    let recipient_1_key_pair = UnsignedEncryptionKeyPair::new(recipient_1_pk, recipient_1_sk);

    let recipient_2_pk = PublicEncryptionKey::<Test>::new(x25519_dalek::PublicKey::from(
        include_bytes!("vectors/multi_anonymous_box/03_recipient_2_pk").to_owned(),
    ));
    let recipient_2_sk = SecretEncryptionKey::new(X25519SecretKey::from(
        include_bytes!("vectors/multi_anonymous_box/04_recipient_2_sk").to_owned(),
    ));

    let recipient_2_key_pair = UnsignedEncryptionKeyPair::new(recipient_2_pk, recipient_2_sk);

    let message = include_bytes!("vectors/multi_anonymous_box/05_message").to_vec();
    let multi_anonymous_box: MultiAnonymousBox<Vec<u8>, NUM_RECIPIENTS> =
        MultiAnonymousBox::from_vec_unchecked(
            include_bytes!("vectors/multi_anonymous_box/06_multi_anonymous_box").to_vec(),
        );

    // ensure for both recipients that decryption results in the original message
    let actual_1 = MultiAnonymousBox::decrypt(&recipient_1_key_pair, &multi_anonymous_box)?;
    assert_eq!(message, actual_1);
    let actual_2 = MultiAnonymousBox::decrypt(&recipient_2_key_pair, &multi_anonymous_box)?;
    assert_eq!(message, actual_2);

    Ok(())
}

#[test]
fn test_journalist_dead_drop() -> anyhow::Result<()> {
    let journalist_pk =
        PublicEncryptionKey::<JournalistMessaging>::new(x25519_dalek::PublicKey::from(
            include_bytes!("vectors/journalist_dead_drop/01_journalist_pk").to_owned(),
        ));
    // Create a signed key with an invalid, but unchecked, certificate.
    let journalist_pk = JournalistMessagingPublicKey::new(
        journalist_pk,
        Signature::from_vec_unchecked(vec![0; 64]),
        time::now(),
    );
    let journalist_sk = SecretEncryptionKey::new(X25519SecretKey::from(
        include_bytes!("vectors/journalist_dead_drop/02_journalist_sk").to_owned(),
    ));

    let journalist_msg_key_pair = SignedEncryptionKeyPair::new(journalist_pk, journalist_sk);
    let journalist_msg_key_pairs = [journalist_msg_key_pair];

    let user_pk = UserPublicKey::new(x25519_dalek::PublicKey::from(
        include_bytes!("vectors/journalist_dead_drop/03_user_pk").to_owned(),
    ));

    let covernode_pk =
        PublicEncryptionKey::<CoverNodeMessaging>::new(x25519_dalek::PublicKey::from(
            include_bytes!("vectors/journalist_dead_drop/04_covernode_pk").to_owned(),
        ));

    // As with the journalist public key, reate a signed key with an invalid, but unchecked, certificate.
    let covernode_msg_pk = CoverNodeMessagingPublicKey::new(
        covernode_pk,
        Signature::from_vec_unchecked(vec![0; 64]),
        time::now(),
    );

    let covernode_msg_pks = [&covernode_msg_pk];

    // This dead drop includes 10 messages, 5 of which are encrypted with the
    // journalist's public key saved on disk, and the remaining 5 with another
    // journalist's public key not saved on disk.
    let dead_drop = SerializedUserToJournalistDeadDropMessages::from_vec_unchecked(
        include_bytes!("vectors/journalist_dead_drop/05_journalist_dead_drop").to_vec(),
    );

    let mut messages = dead_drop.deserialize().messages;

    assert_eq!(messages.len(), 10);

    for i in 0..5 {
        let message = messages.pop().unwrap();
        let message = get_decrypted_journalist_dead_drop_message(
            &covernode_msg_pks,
            &journalist_msg_key_pairs,
            &message,
            i,
        );

        assert_eq!(
            message, None,
            "expected decryption to fail, but got: {message:?}"
        );
    }

    for i in 0..5 {
        let message = messages.pop().unwrap();
        let u2j_message_with_dead_drop_id = get_decrypted_journalist_dead_drop_message(
            &covernode_msg_pks,
            &journalist_msg_key_pairs,
            &message,
            i,
        )
        .unwrap();

        assert_eq!(u2j_message_with_dead_drop_id.u2j_message.reply_key, user_pk);
        assert_eq!(
            u2j_message_with_dead_drop_id
                .u2j_message
                .message
                .to_string()?,
            "こんにちは"
        );
    }

    assert_eq!(messages.len(), 0);

    Ok(())
}

#[test]
fn test_signature() -> anyhow::Result<()> {
    let key_pair = ed25519_dalek::SigningKey::from_bytes(
        include_bytes!("vectors/signature/02_sk")
            .as_ref()
            .try_into()
            .unwrap(),
    );

    let key_pair = UnsignedSigningKeyPair::new(
        PublicSigningKey::<Test>::new(key_pair.verifying_key()),
        key_pair,
    );

    let message = include_bytes!("vectors/signature/03_message").to_vec();
    let signature =
        Signature::from_vec_unchecked(include_bytes!("vectors/signature/04_signature").to_vec());

    traits::PublicSigningKey::verify(key_pair.public_key(), &message, &signature, Utc::now())
        .expect("Signature verified");

    Ok(())
}

#[test]
fn test_certificate_data() -> anyhow::Result<()> {
    let pk = PublicEncryptionKey::<Test>::new(x25519_dalek::PublicKey::from(
        include_bytes!("vectors/certificate_data/01_pk").to_owned(),
    ));

    let not_valid_after_string = std::str::from_utf8(include_bytes!(
        "vectors/certificate_data/02_not_valid_after"
    ))
    .unwrap();
    let not_valid_after =
        DateTime::parse_from_rfc3339(not_valid_after_string).expect("Parse datetime");

    let timestamp_bytes = include_bytes!("vectors/certificate_data/03_timestamp_bytes");
    assert_eq!(timestamp_bytes, &not_valid_after.timestamp().to_be_bytes());

    let expected_certificate_data = KeyCertificateData(
        include_bytes!("vectors/certificate_data/04_certificate_data").to_owned(),
    );
    let actual_certificate_data =
        KeyCertificateData::new_for_encryption_key(&pk.key, not_valid_after.with_timezone(&Utc));

    assert_eq!(
        expected_certificate_data.as_signable_bytes(),
        actual_certificate_data.as_signable_bytes()
    );
    Ok(())
}
