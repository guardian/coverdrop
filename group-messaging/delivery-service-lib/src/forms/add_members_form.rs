use crate::tls_serialized::TlsSerialized;
use chrono::{DateTime, Utc};
use common::{
    api::models::journalist_id::JournalistIdentity,
    form::Form,
    protocol::{keys::JournalistIdKeyPair, roles::JournalistId},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AddMembersFormBody {
    /// TLS-serialized Welcome message for new members
    pub welcome_message: TlsSerialized,
    /// TLS-serialized Commit message for existing members
    pub commit_message: TlsSerialized,
    /// List of existing member client IDs who need the commit message
    pub existing_members: Vec<JournalistIdentity>,
    /// List of new member client IDs who need the welcome message
    pub new_members: Vec<JournalistIdentity>,
}

/// Form for adding new members to an MLS group.
/// Used to distribute Welcome messages to new members and Commit messages to existing members in a single request + transaction.
#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct AddMembersForm(Form<AddMembersFormBody, JournalistId>);

impl AddMembersForm {
    pub fn new(
        welcome_message: TlsSerialized,
        commit_message: TlsSerialized,
        existing_members: Vec<JournalistIdentity>,
        new_members: Vec<JournalistIdentity>,
        signing_key_pair: &JournalistIdKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        let body = AddMembersFormBody {
            welcome_message,
            commit_message,
            existing_members,
            new_members,
        };
        let form = Form::new_from_form_data(body, signing_key_pair, now)?;
        Ok(Self(form))
    }
}

impl std::ops::Deref for AddMembersForm {
    type Target = Form<AddMembersFormBody, JournalistId>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
