use serde::{Deserialize, Serialize};
use sqlx::{Decode, Encode};
use strum::AsRefStr;
use ts_rs::TS;

use crate::{
    api::models::journalist_id::JournalistIdentity, protocol::recipient_tag::RecipientTag,
};

#[derive(
    Clone,
    Serialize,
    Deserialize,
    Encode,
    Decode,
    AsRefStr,
    Debug,
    strum::Display,
    strum::EnumString,
    PartialEq,
    TS,
)]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum JournalistStatus {
    Visible,
    HiddenFromUi,
    HiddenFromResponse,
}

#[derive(Clone, Serialize, Deserialize, TS)]
#[ts(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct JournalistProfile {
    pub id: JournalistIdentity,
    pub display_name: String,
    pub sort_name: String,
    pub description: String,
    pub is_desk: bool,
    pub tag: RecipientTag,
    pub status: JournalistStatus,
}

impl JournalistProfile {
    pub fn new(
        id: JournalistIdentity,
        display_name: String,
        sort_name: String,
        description: String,
        is_desk: bool,
        status: JournalistStatus,
    ) -> Self {
        let tag = RecipientTag::from_journalist_id(&id);

        Self {
            id,
            display_name,
            sort_name,
            description,
            is_desk,
            tag,
            status,
        }
    }
}
