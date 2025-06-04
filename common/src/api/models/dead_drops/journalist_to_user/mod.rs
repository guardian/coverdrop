mod journalist_to_user_dead_drop;
mod journalist_to_user_dead_drop_messages;
mod journalist_to_user_dead_drop_signature_data_v2;
mod serialized_journalist_to_user_dead_drop_messages;
mod unpublished_journalist_to_user_dead_drop;
mod unverified_journalist_to_user_dead_drop;
mod unverified_journalist_to_user_dead_drop_list;

pub use journalist_to_user_dead_drop::JournalistToUserDeadDrop;
pub use journalist_to_user_dead_drop_messages::JournalistToUserDeadDropMessages;
pub use journalist_to_user_dead_drop_signature_data_v2::JournalistToUserDeadDropSignatureDataV2;
pub use serialized_journalist_to_user_dead_drop_messages::SerializedJournalistToUserDeadDropMessages;
pub use unpublished_journalist_to_user_dead_drop::UnpublishedJournalistToUserDeadDrop;
pub use unverified_journalist_to_user_dead_drop::UnverifiedJournalistToUserDeadDrop;
pub use unverified_journalist_to_user_dead_drop_list::UnverifiedJournalistToUserDeadDropsList;

#[cfg(test)]
mod tests_journalist_to_user {
    use journalist_to_user_dead_drop_messages::JournalistToUserDeadDropMessages;

    use super::*;
    use crate::{
        crypto::TwoPartyBox, protocol::constants::JOURNALIST_TO_USER_ENCRYPTED_MESSAGE_LEN,
    };

    #[test]
    fn dead_drop_serde_roundtrip() {
        let messages = vec![
            TwoPartyBox::from_vec_unchecked(vec![1; JOURNALIST_TO_USER_ENCRYPTED_MESSAGE_LEN]),
            TwoPartyBox::from_vec_unchecked(vec![2; JOURNALIST_TO_USER_ENCRYPTED_MESSAGE_LEN]),
            TwoPartyBox::from_vec_unchecked(vec![3; JOURNALIST_TO_USER_ENCRYPTED_MESSAGE_LEN]),
            TwoPartyBox::from_vec_unchecked(vec![4; JOURNALIST_TO_USER_ENCRYPTED_MESSAGE_LEN]),
        ];

        let dead_drop = JournalistToUserDeadDropMessages::new(messages);
        let serialized = dead_drop.serialize();
        let deserialized = serialized.deserialize();
        assert_eq!(dead_drop, deserialized);
    }
}
