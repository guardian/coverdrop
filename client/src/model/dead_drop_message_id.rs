use std::fmt::Display;

use common::api::models::dead_drops::DeadDropId;

#[derive(Hash, Eq, PartialEq)]
pub struct DeadDropMessageId {
    pub dead_drop_id: DeadDropId,
    pub message_index: usize,
}

impl Display for DeadDropMessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}-{}", self.dead_drop_id, self.message_index))
    }
}
