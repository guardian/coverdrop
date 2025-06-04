pub mod covernode_to_journalist_message;
pub mod journalist_to_covernode_message;
pub mod journalist_to_user_message;
pub mod user_to_covernode_message;
pub mod user_to_journalist_message;
pub mod user_to_journalist_message_with_dead_drop_id;

use crate::protocol::constants::RECIPIENT_TAG_LEN;
use serde::{Deserialize, Serialize};

//
// Constants for [JournalistToCoverNodeMessage]
//
pub const FLAG_J2U_COVER: u8 = 0x00;
pub const FLAG_J2U_REAL: u8 = 0x01;

//
// Constants for [JournalistToUserMessage]
//
pub const FLAG_J2U_MESSAGE_TYPE_MESSAGE: u8 = 0x00;
pub const FLAG_J2U_MESSAGE_TYPE_HANDOVER: u8 = 0x01;

//
// Constants for [UserToCoverNodeMessage]
//
pub const RECIPIENT_TAG_BYTES_U2J_COVER: [u8; RECIPIENT_TAG_LEN] = [0_u8; RECIPIENT_TAG_LEN];

//
// API-level messages
//

pub type MessageId = i32;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(transparent, deny_unknown_fields)]
pub struct Message<M> {
    pub data: M,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PublishedMessage<M> {
    pub id: MessageId,
    pub data: M,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PublishedMessagesList<M> {
    pub messages: Vec<PublishedMessage<M>>,
}
