use common::api::models::journalist_id::JournalistIdentity;
use common::api::models::messages::journalist_to_user_message::JournalistToUserMessage;

/// Used by the client to process which messages have been sent to a specific user.
pub struct JournalistToUserDeadDropMessage {
    /// The journalist identity associated with this message, discovered by attempting to decrypt
    /// with all the candiate keys
    pub journalist_id: JournalistIdentity,
    pub message: JournalistToUserMessage,
}

impl JournalistToUserDeadDropMessage {
    pub fn new(journalist_id: JournalistIdentity, message: JournalistToUserMessage) -> Self {
        Self {
            journalist_id,
            message,
        }
    }
}
