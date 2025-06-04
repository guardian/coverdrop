use chrono::{DateTime, Utc};
use common::api::models::dead_drops::UnverifiedUserToJournalistDeadDropsList;
use common::api::models::messages::user_to_journalist_message_with_dead_drop_id::UserToJournalistMessageWithDeadDropId;
use common::protocol::covernode::verify_user_to_journalist_dead_drop_list;
use common::protocol::journalist::get_decrypted_journalist_dead_drop_message;
use common::protocol::keys::CoverDropPublicKeyHierarchy;
use journalist_vault::JournalistVault;

/// Pulls dead drops from the server and attempts to decrypt all the messages using the journalist's messaging keys
pub async fn load_journalist_dead_drop_messages(
    dead_drop_list: UnverifiedUserToJournalistDeadDropsList,
    keys: &CoverDropPublicKeyHierarchy,
    vault: &JournalistVault,
    now: DateTime<Utc>,
) -> anyhow::Result<usize> {
    let Some(max_id) = dead_drop_list.max_id() else {
        return Ok(0);
    };

    let verified_dead_drop_list =
        verify_user_to_journalist_dead_drop_list(keys, dead_drop_list, now);

    let journalist_msg_key_pairs = vault
        .msg_key_pairs_for_decryption(now)
        .await?
        .collect::<Vec<_>>();

    let covernode_msg_pks = keys
        .covernode_msg_pk_iter()
        .map(|(_, msg_pk)| msg_pk)
        .collect::<Vec<_>>();

    let decrypted_messages: Vec<UserToJournalistMessageWithDeadDropId> = verified_dead_drop_list
        .iter()
        .flat_map(|dead_drop| {
            dead_drop
                .data
                .messages
                .iter()
                .filter_map(|encrypted_message| {
                    get_decrypted_journalist_dead_drop_message(
                        &covernode_msg_pks,
                        &journalist_msg_key_pairs,
                        encrypted_message,
                        dead_drop.id,
                    )
                })
        })
        .collect();

    let messages_loaded = decrypted_messages.len();
    vault
        .add_messages_from_user_to_journalist_and_update_max_dead_drop_id(
            &decrypted_messages,
            max_id,
            now,
        )
        .await?;

    Ok(messages_loaded)
}
