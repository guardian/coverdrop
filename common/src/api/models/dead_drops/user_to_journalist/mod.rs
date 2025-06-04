mod serialized_user_to_journalist_dead_drop_messages;
mod unpublished_user_to_journalist_dead_drop;
mod unverified_user_to_journalist_dead_drop;
mod unverified_user_to_journalist_dead_drop_list;
mod user_to_journalist_dead_drop;
mod user_to_journalist_dead_drop_certificate_data_v1;
mod user_to_journalist_dead_drop_messages;
mod user_to_journalist_dead_drop_signature_data_v2;

pub use serialized_user_to_journalist_dead_drop_messages::SerializedUserToJournalistDeadDropMessages;
pub use unpublished_user_to_journalist_dead_drop::UnpublishedUserToJournalistDeadDrop;
pub use unverified_user_to_journalist_dead_drop::UnverifiedUserToJournalistDeadDrop;
pub use unverified_user_to_journalist_dead_drop_list::UnverifiedUserToJournalistDeadDropsList;
pub use user_to_journalist_dead_drop::UserToJournalistDeadDrop;
pub use user_to_journalist_dead_drop_certificate_data_v1::UserToJournalistDeadDropCertificateDataV1;
pub use user_to_journalist_dead_drop_messages::UserToJournalistDeadDropMessages;
pub use user_to_journalist_dead_drop_signature_data_v2::UserToJournalistDeadDropSignatureDataV2;

#[cfg(test)]
mod tests_user_to_journalist {
    use super::*;
    use crate::{
        crypto::TwoPartyBox, protocol::constants::COVERNODE_TO_JOURNALIST_ENCRYPTED_MESSAGE_LEN,
    };

    #[test]
    fn dead_drop_serde_round_trip() {
        let messages = vec![
            TwoPartyBox::from_vec_unchecked(vec![1; COVERNODE_TO_JOURNALIST_ENCRYPTED_MESSAGE_LEN]),
            TwoPartyBox::from_vec_unchecked(vec![2; COVERNODE_TO_JOURNALIST_ENCRYPTED_MESSAGE_LEN]),
            TwoPartyBox::from_vec_unchecked(vec![3; COVERNODE_TO_JOURNALIST_ENCRYPTED_MESSAGE_LEN]),
            TwoPartyBox::from_vec_unchecked(vec![4; COVERNODE_TO_JOURNALIST_ENCRYPTED_MESSAGE_LEN]),
        ];

        let dead_drop = UserToJournalistDeadDropMessages::new(messages);
        let serialized = dead_drop.serialize();
        let deserialized = serialized.deserialize();
        assert_eq!(dead_drop, deserialized);
    }
}
