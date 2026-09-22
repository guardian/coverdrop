use chrono::{DateTime, Utc};
use common::api::models::dead_drops::UnverifiedJournalistToUserDeadDropsList;
use common::api::models::messages::journalist_to_user_message::JournalistToUserMessage;
use common::client::mailbox::user_mailbox::UserMailbox;
use common::protocol::covernode::verify_journalist_to_user_dead_drop_list;
use common::protocol::keys::CoverDropPublicKeyHierarchy;
use common::protocol::user::get_decrypted_user_dead_drop_message;

pub fn load_user_dead_drop_messages(
    dead_drop_list: &UnverifiedJournalistToUserDeadDropsList,
    keys: &CoverDropPublicKeyHierarchy,
    mailbox: &mut UserMailbox,
    now: DateTime<Utc>,
) -> anyhow::Result<usize> {
    let mut messages_loaded = 0;

    let created_after = mailbox.max_dead_drop_created_at();

    let verified_dead_drops =
        verify_journalist_to_user_dead_drop_list(keys, dead_drop_list, created_after, now);

    // The cursors must only ever be advanced using verified dead drops, otherwise a malicious
    // API could poison them with an unverifiable dead drop with a far future `created_at`.
    let max_dead_drop_created_at = verified_dead_drops
        .iter()
        .map(|d| d.created_at)
        .max()
        .unwrap_or(created_after);

    // POSSIBLE IMPROVEMENT:
    // We could keep a track of who the user has messages so we don't have to check every single key
    // in the verified keys list.

    for dead_drop in verified_dead_drops {
        for msg in dead_drop.data.messages {
            if let Ok(Some((journalist_id, message))) =
                get_decrypted_user_dead_drop_message(mailbox.user_key_pair(), keys, &msg)
            {
                match message {
                    JournalistToUserMessage::Message(message) => {
                        mailbox.add_message_to_user_from_journalist(&journalist_id, &message);
                    }
                }
                messages_loaded += 1;
            }
        }
    }

    mailbox.set_max_dead_drop_created_at(max_dead_drop_created_at);

    Ok(messages_loaded)
}
