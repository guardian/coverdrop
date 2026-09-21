use common::{
    api::models::dead_drops::{JournalistToUserDeadDropMessages, UserToJournalistDeadDropMessages},
    aws::kinesis::models::checkpoint::CheckpointsJson,
    epoch::Epoch,
};
use covernode_database::MessageHashesWithExpiries;

#[derive(Debug)]
pub struct UserToJournalistDeadDropContentWithCheckpointsAndMessageHashes {
    pub dead_drop_content: UserToJournalistDeadDropMessages,
    pub checkpoints_json: CheckpointsJson,
    pub message_hashes: MessageHashesWithExpiries,
    pub encryption_max_epoch: Epoch,
}

#[derive(Debug)]
pub struct JournalistToUserDeadDropContentWithCheckpointsAndMessageHashes {
    pub dead_drop_content: JournalistToUserDeadDropMessages,
    pub checkpoints_json: CheckpointsJson,
    pub message_hashes: MessageHashesWithExpiries,
}
