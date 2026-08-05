use chrono::{DateTime, Utc};
use common::api::models::sentinel_id::SentinelIdentity;
use journalist_vault::{GroupId as VaultGroupId, GroupMessage, GroupMessageContent, MessageId};
use openmls::{
    framing::ApplicationMessage,
    prelude::{BasicCredential, Credential, GroupId as OpenMlsGroupId, LeafNodeIndex},
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub struct SentinelIdentityWithLeafIndex {
    pub identity: SentinelIdentity,
    pub leaf_index: LeafNodeIndex,
}

pub trait SentinelIdentityExt {
    fn from_mls_credential(credential: Credential) -> anyhow::Result<SentinelIdentity>;
}

impl SentinelIdentityExt for SentinelIdentity {
    fn from_mls_credential(credential: Credential) -> anyhow::Result<SentinelIdentity> {
        let basic_cred = BasicCredential::try_from(credential)
            .map_err(|_| anyhow::anyhow!("Failed to parse credential as BasicCredential"))?;
        let identity_str = String::from_utf8_lossy(basic_cred.identity()).to_string();
        SentinelIdentity::new(&identity_str).map_err(Into::into)
    }
}

pub trait GroupIdExt {
    fn to_open_mls_group_id(&self) -> OpenMlsGroupId;

    fn from_open_mls_group_id(group_id: &OpenMlsGroupId) -> anyhow::Result<Self>
    where
        Self: Sized;
}

impl GroupIdExt for VaultGroupId {
    fn to_open_mls_group_id(&self) -> OpenMlsGroupId {
        OpenMlsGroupId::from_slice(self.to_string().as_bytes())
    }

    fn from_open_mls_group_id(group_id: &OpenMlsGroupId) -> anyhow::Result<Self> {
        let group_id_str = String::from_utf8(group_id.as_slice().to_vec())?;
        Ok(Self::new(group_id_str))
    }
}

pub trait GroupMessageExt {
    fn from_mls_application_message(
        message_bytes: ApplicationMessage,
        group_id: &OpenMlsGroupId,
        sender: SentinelIdentity,
        published_at: DateTime<Utc>,
    ) -> anyhow::Result<GroupMessage>;
}

impl GroupMessageExt for GroupMessage {
    fn from_mls_application_message(
        application_message: ApplicationMessage,
        group_id: &OpenMlsGroupId,
        sender: SentinelIdentity,
        published_at: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        let message_bytes = application_message.into_bytes();
        let mls_content_with_id = MlsMessageContentWithId::from_bytes(&message_bytes)?;
        let content: GroupMessageContent = mls_content_with_id.content.into();

        let group_message = GroupMessage::new(
            mls_content_with_id.id,
            sender,
            VaultGroupId::from_open_mls_group_id(group_id)?,
            content,
            false,
            published_at,
        );
        Ok(group_message)
    }
}

/// The object passed from Sentinel's backend to the frontend with group information,
/// group members, and all group messages.
#[derive(TS)]
#[ts(export)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Group {
    id: VaultGroupId,
    display_name: String,
    description: String,
    members: Vec<SentinelIdentity>,
    messages: Vec<GroupMessage>,
}

impl Group {
    pub fn new(
        id: VaultGroupId,
        display_name: String,
        description: String,
        members: Vec<SentinelIdentity>,
        messages: Vec<GroupMessage>,
    ) -> Self {
        Self {
            id,
            display_name,
            description,
            members,
            messages,
        }
    }

    pub fn id(&self) -> &VaultGroupId {
        &self.id
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn messages(&self) -> &[GroupMessage] {
        &self.messages
    }

    pub fn members(&self) -> &[SentinelIdentity] {
        &self.members
    }
}

/// The content of a message that is encrypted via MLS and sent over the delivery service.
/// This is a subset of `GroupMessageContent` — it excludes variants like `UsersAdded`
/// and `UsersRemoved` which are only created locally and never sent over the wire.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum MlsMessageContent {
    /// Message content containing text to be displayed in the group chat.
    Text(String),
    /// Indicates that a user has started or stopped typing in a group chat.
    /// Clients give these a short TTL to account for clients who go offline after sending one.
    IsTyping(bool),
    /// Indicates that the group name has changed. The new group name is included in the message.
    GroupNameChanged(String),
    /// Indicates that the group description has changed. The new group description is included in the message.
    GroupDescriptionChanged(String),
    /// Initial group info message sent to new members of a group.
    GroupInfo {
        display_name: String,
        description: String,
    },
}

impl From<MlsMessageContent> for GroupMessageContent {
    fn from(mls: MlsMessageContent) -> Self {
        match mls {
            MlsMessageContent::Text(s) => GroupMessageContent::Text(s),
            MlsMessageContent::IsTyping(b) => GroupMessageContent::IsTyping(b),
            MlsMessageContent::GroupNameChanged(s) => GroupMessageContent::GroupNameChanged(s),
            MlsMessageContent::GroupDescriptionChanged(s) => {
                GroupMessageContent::GroupDescriptionChanged(s)
            }
            MlsMessageContent::GroupInfo {
                display_name,
                description,
            } => GroupMessageContent::GroupInfo {
                display_name,
                description,
            },
        }
    }
}

/// This is the content of the private messages encrypted for the MLS group
/// and sent over the delivery service. It wraps `MlsMessageContent` and adds a unique id to it.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct MlsMessageContentWithId {
    pub id: MessageId,
    pub content: MlsMessageContent,
}

impl MlsMessageContentWithId {
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_cbor::Error> {
        serde_cbor::to_vec(self)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_cbor::Error> {
        serde_cbor::from_slice(bytes)
    }

    /// Generates a new `MlsMessageContentWithId` with a unique id.
    pub fn new_from_content(content: MlsMessageContent) -> Self {
        let id = MessageId::new();
        Self { id, content }
    }
}
