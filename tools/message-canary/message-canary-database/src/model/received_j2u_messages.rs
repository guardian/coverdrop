use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub struct AllReceivedJournalistToUserMessages(HashMap<i32, HashSet<String>>);

impl AllReceivedJournalistToUserMessages {
    pub fn get_messages_for_user_id(&self, user_id: i32) -> Option<&HashSet<String>> {
        self.0.get(&user_id)
    }

    pub fn insert_user_messages(&mut self, user_id: i32, messages: Vec<String>) {
        let message_set: HashSet<String> = messages.into_iter().collect();

        self.0.insert(user_id, message_set);
    }
}
