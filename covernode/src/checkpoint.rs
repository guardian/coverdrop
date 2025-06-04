use common::{
    api::models::dead_drops::{JournalistToUserDeadDropMessages, UserToJournalistDeadDropMessages},
    aws::kinesis::models::checkpoint::CheckpointsJson,
    epoch::Epoch,
};

#[derive(Debug)]
pub struct UserToJournalistDeadDropContentWithCheckpoints {
    pub dead_drop_content: UserToJournalistDeadDropMessages,
    pub checkpoints_json: CheckpointsJson,
    pub encryption_max_epoch: Epoch,
}

#[derive(Debug)]
pub struct JournalistToUserDeadDropContentWithCheckpoints {
    pub dead_drop_content: JournalistToUserDeadDropMessages,
    pub checkpoints_json: CheckpointsJson,
}
