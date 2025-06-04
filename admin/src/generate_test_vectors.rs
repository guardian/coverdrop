use common::api::models::messages::user_to_journalist_message::*;
use common::crypto::keys::encryption::UnsignedEncryptionKeyPair;
use common::crypto::keys::role::Test;
use common::crypto::keys::signing::traits::PublicSigningKey;
use common::crypto::keys::signing::UnsignedSigningKeyPair;
use common::crypto::{AnonymousBox, MultiAnonymousBox, Signable, TwoPartyBox};
use common::FixedSizeMessageText;

use chrono::{TimeZone, Utc};
use common::api::models::dead_drops::UserToJournalistDeadDropMessages;
use common::api::models::messages::covernode_to_journalist_message::{
    CoverNodeToJournalistMessage, EncryptedCoverNodeToJournalistMessage,
};
use common::crypto::keys::key_certificate_data::KeyCertificateData;
use common::protocol::roles::{CoverNodeMessaging, JournalistMessaging, User};
use rand::RngCore;
use std::path::Path;
use std::{fs, io};

pub fn generate_test_vectors(path: &Path) -> anyhow::Result<()> {
    generate_test_vectors_anonymous_box(path.join("anonymous_box").as_path())?;
    generate_test_vectors_two_party_box(path.join("two_party_box").as_path())?;
    generate_test_vectors_multi_anonymous_box(path.join("multi_anonymous_box").as_path())?;
    generate_test_vectors_journalist_dead_drop(path.join("journalist_dead_drop").as_path())?;
    generate_test_vectors_signature(path.join("signature").as_path())?;
    generate_test_vectors_certificate_data(path.join("certificate_data").as_path())?;
    Ok(())
}

fn generate_test_vectors_anonymous_box(path: &Path) -> anyhow::Result<()> {
    ensure_dir(path);
    println!("Creating anonymous box test vectors in {:?}", &path);

    let recipient_key_pair = UnsignedEncryptionKeyPair::<JournalistMessaging>::generate();

    write_bytes(
        path,
        "01_recipient_pk",
        &recipient_key_pair.raw_public_key().to_bytes(),
    )?;
    write_bytes(
        path,
        "02_recipient_sk",
        &recipient_key_pair.secret_key().to_bytes(),
    )?;

    let message = generate_random_message();
    write_bytes(path, "03_message", &message)?;

    let anonymous_box = AnonymousBox::encrypt(recipient_key_pair.public_key(), message)?;
    write_bytes(path, "04_anonymous_box", anonymous_box.as_bytes())?;

    Ok(())
}

fn generate_test_vectors_two_party_box(path: &Path) -> anyhow::Result<()> {
    ensure_dir(path);
    println!("Creating two party box test vectors in {:?}", &path);

    let sender_key_pair = UnsignedEncryptionKeyPair::<Test>::generate();
    write_bytes(
        path,
        "01_sender_pk",
        &sender_key_pair.raw_public_key().to_bytes(),
    )?;
    write_bytes(
        path,
        "02_sender_sk",
        &sender_key_pair.secret_key().to_bytes(),
    )?;

    let recipient_key_pair = UnsignedEncryptionKeyPair::<Test>::generate();
    write_bytes(
        path,
        "03_recipient_pk",
        &recipient_key_pair.raw_public_key().to_bytes(),
    )?;
    write_bytes(
        path,
        "04_recipient_sk",
        &recipient_key_pair.secret_key().to_bytes(),
    )?;

    let message = generate_random_message();
    write_bytes(path, "05_message", &message)?;

    let two_party_box = TwoPartyBox::encrypt(
        recipient_key_pair.public_key(),
        sender_key_pair.secret_key(),
        message,
    )?;
    write_bytes(path, "06_two_party_box", two_party_box.as_bytes())?;

    Ok(())
}

fn generate_test_vectors_multi_anonymous_box(path: &Path) -> anyhow::Result<()> {
    ensure_dir(path);
    println!("Creating multi anonymous box test vectors in {:?}", &path);

    let recipient_1_key_pair = UnsignedEncryptionKeyPair::<JournalistMessaging>::generate();
    write_bytes(
        path,
        "01_recipient_1_pk",
        &recipient_1_key_pair.raw_public_key().to_bytes(),
    )?;
    write_bytes(
        path,
        "02_recipient_1_sk",
        &recipient_1_key_pair.secret_key().to_bytes(),
    )?;

    let recipient_2_key_pair = UnsignedEncryptionKeyPair::<JournalistMessaging>::generate();
    write_bytes(
        path,
        "03_recipient_2_pk",
        &recipient_2_key_pair.raw_public_key().to_bytes(),
    )?;
    write_bytes(
        path,
        "04_recipient_2_sk",
        &recipient_2_key_pair.secret_key().to_bytes(),
    )?;

    let message = generate_random_message();
    write_bytes(path, "05_message", &message)?;

    let recipients_pks = [
        recipient_1_key_pair.public_key(),
        recipient_2_key_pair.public_key(),
    ];
    let anonymous_box = MultiAnonymousBox::encrypt(recipients_pks, message)?;
    write_bytes(path, "06_multi_anonymous_box", anonymous_box.as_bytes())?;

    Ok(())
}

fn generate_test_vectors_journalist_dead_drop(path: &Path) -> anyhow::Result<()> {
    ensure_dir(path);
    println!("Creating journalist dead drop test vectors in {:?}", &path);

    let journalist_key_pair = UnsignedEncryptionKeyPair::<JournalistMessaging>::generate();
    let other_journalist_key_pair = UnsignedEncryptionKeyPair::<JournalistMessaging>::generate();
    let user_key_pair = UnsignedEncryptionKeyPair::<User>::generate();
    let covernode_msg_key_pair = UnsignedEncryptionKeyPair::<CoverNodeMessaging>::generate();

    write_bytes(
        path,
        "01_journalist_pk",
        &journalist_key_pair.raw_public_key().to_bytes(),
    )?;
    write_bytes(
        path,
        "02_journalist_sk",
        &journalist_key_pair.secret_key().to_bytes(),
    )?;
    write_bytes(
        path,
        "03_user_pk",
        &user_key_pair.raw_public_key().to_bytes(),
    )?;
    write_bytes(
        path,
        "04_covernode_pk",
        &covernode_msg_key_pair.raw_public_key().to_bytes(),
    )?;

    let message = FixedSizeMessageText::new("こんにちは")?;

    let mut messages: Vec<EncryptedCoverNodeToJournalistMessage> = vec![];

    for _ in 0..5 {
        let inner_message =
            UserToJournalistMessage::new(message.clone(), user_key_pair.public_key());
        let inner_message = inner_message.serialize();
        let inner_message = EncryptedUserToJournalistMessage::encrypt(
            journalist_key_pair.public_key(),
            inner_message,
        )?;

        let message = CoverNodeToJournalistMessage::new(inner_message);
        let message = message.serialize();
        let message = EncryptedCoverNodeToJournalistMessage::encrypt(
            journalist_key_pair.public_key(),
            covernode_msg_key_pair.secret_key(),
            message,
        )?;
        messages.push(message);
    }

    for _ in 0..5 {
        let inner_message =
            UserToJournalistMessage::new(message.clone(), user_key_pair.public_key());
        let inner_message = inner_message.serialize();
        let inner_message = EncryptedUserToJournalistMessage::encrypt(
            other_journalist_key_pair.public_key(),
            inner_message,
        )?;

        let message = CoverNodeToJournalistMessage::new(inner_message);
        let message = message.serialize();
        let message = EncryptedCoverNodeToJournalistMessage::encrypt(
            journalist_key_pair.public_key(),
            covernode_msg_key_pair.secret_key(),
            message,
        )?;
        messages.push(message);
    }

    let dead_drop = UserToJournalistDeadDropMessages::new(messages);
    let dead_drop = dead_drop.serialize();

    write_bytes(path, "05_journalist_dead_drop", dead_drop.as_bytes())?;

    Ok(())
}

fn generate_test_vectors_signature(path: &Path) -> anyhow::Result<()> {
    ensure_dir(path);
    println!("Creating signature test vectors in {:?}", &path);

    let key_pair = UnsignedSigningKeyPair::<Test>::generate();
    write_bytes(path, "01_pk", key_pair.public_key().as_bytes())?;
    write_bytes(path, "02_sk", &key_pair.secret_key.to_bytes()[..])?;

    let message = generate_random_message();
    write_bytes(path, "03_message", &message)?;

    let signature = key_pair.sign(&message);
    write_bytes(path, "04_signature", &signature.to_bytes()[..])?;

    Ok(())
}

fn generate_test_vectors_certificate_data(path: &Path) -> anyhow::Result<()> {
    ensure_dir(path);
    println!("Creating certificate data test vectors in {:?}", &path);

    let key_pair = UnsignedEncryptionKeyPair::<Test>::generate();
    write_bytes(path, "01_pk", &key_pair.raw_public_key().to_bytes())?;

    let not_valid_after = Utc.with_ymd_and_hms(2023, 2, 24, 13, 37, 42).unwrap();
    write_bytes(
        path,
        "02_not_valid_after",
        not_valid_after.to_rfc3339().as_bytes(),
    )?;

    let timestamp_bytes = not_valid_after.timestamp().to_be_bytes();
    write_bytes(path, "03_timestamp_bytes", &timestamp_bytes)?;

    let certificate_data =
        KeyCertificateData::new_for_encryption_key(&key_pair.raw_public_key(), not_valid_after);
    write_bytes(
        path,
        "04_certificate_data",
        certificate_data.as_signable_bytes(),
    )?;

    Ok(())
}

const LEN_MESSAGE: usize = 997;

fn generate_random_message() -> Vec<u8> {
    let mut message: [u8; LEN_MESSAGE] = [0; LEN_MESSAGE];
    let mut rng = rand::thread_rng();
    rng.fill_bytes(&mut message);
    message.to_vec()
}

fn write_bytes(base_path: &Path, filename: &str, bytes: &[u8]) -> io::Result<()> {
    let path = base_path.join(filename);
    fs::write(path.as_path(), bytes)
}

fn ensure_dir(path: &Path) {
    if !path.exists() {
        fs::create_dir_all(path).unwrap();
    }
}
