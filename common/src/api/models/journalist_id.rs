use std::{
    fmt::{self, Display, Formatter},
    ops::Deref,
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::Error;

pub const MAX_JOURNALIST_IDENTITY_LEN: usize = 128;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, sqlx::Type, TS)]
#[serde(transparent, deny_unknown_fields)]
#[ts(type = "string")]
#[sqlx(transparent)]
pub struct JournalistIdentity(String);

impl JournalistIdentity {
    pub fn new(id: &str) -> Result<Self, Error> {
        if !id.is_ascii()
            || id.contains('/')
            || id.contains('\\')
            || id.len() > MAX_JOURNALIST_IDENTITY_LEN
            || id.is_empty()
        {
            Err(Error::InvalidJournalistId)
        } else {
            Ok(JournalistIdentity(id.into()))
        }
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl FromStr for JournalistIdentity {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        JournalistIdentity::new(s)
    }
}

impl Deref for JournalistIdentity {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<String> for JournalistIdentity {
    fn as_ref(&self) -> &String {
        &self.0
    }
}

impl Display for JournalistIdentity {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<JournalistIdentity> for String {
    fn from(value: JournalistIdentity) -> Self {
        value.0
    }
}
