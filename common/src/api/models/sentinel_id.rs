use std::{
    fmt::{self, Display, Formatter},
    ops::Deref,
    str::FromStr,
};

use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::Error;

pub const MAX_SENTINEL_IDENTITY_LEN: usize = 128;

lazy_static! {
    static ref SENTINEL_ID_REGEX: Regex = Regex::new(&format!(
        "^[a-zA-Z0-9_-]{{1,{}}}$",
        MAX_SENTINEL_IDENTITY_LEN
    ))
    .unwrap();
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, sqlx::Type, TS)]
#[serde(transparent, deny_unknown_fields)]
#[ts(type = "string")]
#[sqlx(transparent)]
pub struct SentinelIdentity(String);

impl SentinelIdentity {
    pub fn new(id: &str) -> Result<Self, Error> {
        if !SENTINEL_ID_REGEX.is_match(id) {
            Err(Error::InvalidSentinelId)
        } else {
            Ok(SentinelIdentity(id.into()))
        }
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl FromStr for SentinelIdentity {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        SentinelIdentity::new(s)
    }
}

impl Deref for SentinelIdentity {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<String> for SentinelIdentity {
    fn as_ref(&self) -> &String {
        &self.0
    }
}

impl Display for SentinelIdentity {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<SentinelIdentity> for String {
    fn from(value: SentinelIdentity) -> Self {
        value.0
    }
}
