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
    let Some(max_dead_drop_id) = dead_drop_list.max_id() else {
        return Ok(messages_loaded);
    };

    let verified_dead_drops = verify_journalist_to_user_dead_drop_list(keys, dead_drop_list, now);

    // POSSIBLE IMPROVEMENT:
    // We could keep a track of who the user has messages so we don't have to check every single key
    // in the verified keys list. This is non-trivial since we want to support hand-off of journalists
    // ideally without any client side state.
    //
    // A possible solution to this would be to have a "system" message which a journalist can send to a client
    // the client interprets this as a command rather than as a regular message and adds the journalist to their
    // list of known journalists. Even with this it's not super straight forward since the forwarding command and
    // the first message from another journalist could arrive out of order.

    for dead_drop in verified_dead_drops {
        for msg in dead_drop.data.messages {
            if let Ok(Some((journalist_id, message))) =
                get_decrypted_user_dead_drop_message(mailbox.user_key_pair(), keys, &msg)
            {
                match message {
                    JournalistToUserMessage::Message(message) => {
                        mailbox.add_message_to_user_from_journalist(&journalist_id, &message);
                    }
                    JournalistToUserMessage::HandOver(_) => {
                        // POSSIBLY TODO implement the optimisation that allows users to limit the number of journalist keys they check
                        // against dead drop messages. As it stands the Rust code doesn't do this since we only act as a user for
                        // test and debug purposes.
                    }
                }
                messages_loaded += 1;
            }
        }
    }

    mailbox.set_max_dead_drop_id(max_dead_drop_id);

    Ok(messages_loaded)
}
