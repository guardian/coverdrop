use crate::api::models::dead_drops::DeadDropId;

use super::user_to_journalist_message::UserToJournalistMessage;

/// Wrapper for [UserToJournalistMessage] including the ID of the U2J dead drop
/// which contained it.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct UserToJournalistMessageWithDeadDropId {
    pub u2j_message: UserToJournalistMessage,
    pub dead_drop_id: DeadDropId,
}
