use chrono::Utc;
use common::{
    api::models::{
        journalist_id::JournalistIdentity,
        messages::{
            user_to_journalist_message::UserToJournalistMessage,
            user_to_journalist_message_with_dead_drop_id::UserToJournalistMessageWithDeadDropId,
        },
    },
    crypto::keys::encryption::UnsignedEncryptionKeyPair,
    protocol::{
        keys::{generate_journalist_provisioning_key_pair, generate_organization_key_pair},
        roles::User,
    },
    FixedSizeMessageText,
};
use journalist_vault::{JournalistVault, VaultMessage};
use tempfile::tempdir_in;

#[tokio::test]
async fn message_round_trip() {
    let temp_dir = tempdir_in(std::env::current_dir().unwrap()).unwrap();
    let mut db_path = temp_dir.path().to_owned();
    db_path.push("test.db");

    let user_key_pair = UnsignedEncryptionKeyPair::<User>::generate();

    let now = Utc::now();
    let journalist_id = JournalistIdentity::new("Hello").unwrap();

    let org_key_pair = generate_organization_key_pair(now);

    let journalist_provisioning_key_pair =
        generate_journalist_provisioning_key_pair(&org_key_pair, now);

    let message = FixedSizeMessageText::new("Hello").unwrap();

    let u2j_message_with_dead_drop_id = UserToJournalistMessageWithDeadDropId {
        u2j_message: UserToJournalistMessage::new(
            message.clone(),
            user_key_pair.public_key().clone(),
        ),
        dead_drop_id: 1,
    };

    {
        let anchor_org_pk = org_key_pair.public_key().clone().into_anchor();
        let org_and_journalist_provisioning_pks = vec![(
            anchor_org_pk,
            journalist_provisioning_key_pair.public_key().clone(),
        )];

        let vault = JournalistVault::create(
            &db_path,
            "test_password",
            &journalist_id,
            &org_and_journalist_provisioning_pks,
            now,
        )
        .await
        .expect("Create journalist vault");

        vault
            .add_messages_from_user_to_journalist_and_update_max_dead_drop_id(
                &[u2j_message_with_dead_drop_id],
                1,
                now,
            )
            .await
            .unwrap();
    }

    {
        let vault = JournalistVault::open(&db_path, "test_password")
            .await
            .expect("Load journalist vault");

        let mut messages = vault.messages().await.unwrap();

        assert_eq!(messages.len(), 1);

        let new_message = match messages.pop().unwrap() {
            VaultMessage::U2J(m) => m,
            _ => panic!("Expected a U2J message"),
        };

        assert_eq!(
            new_message.message,
            message.to_string().expect("convert to string")
        );

        assert_eq!(new_message.received_at, now);
    }
}
