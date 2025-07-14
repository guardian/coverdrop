use std::{
    fmt::{self, Display, Formatter},
    ops::Deref,
    str::FromStr,
};

use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::Error;

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, sqlx::Type, PartialOrd, Ord,
)]
#[serde(transparent, deny_unknown_fields)]
#[sqlx(transparent)]
pub struct CoverNodeIdentity(String);

const COVERNODE_ID_REGEX_STR: &str = "covernode_\\d\\d\\d";

lazy_static! {
    static ref COVERNODE_ID_REGEX: Regex = Regex::new(COVERNODE_ID_REGEX_STR).unwrap();
}

impl CoverNodeIdentity {
    pub fn new(s: &str) -> Result<Self, Error> {
        if COVERNODE_ID_REGEX.is_match(s) {
            Ok(CoverNodeIdentity(s.to_string()))
        } else {
            Err(Error::InvalidCoverNodeId {
                expected_pattern: COVERNODE_ID_REGEX_STR,
            })
        }
    }

    pub fn from_node_id(node_number: u8) -> Self {
        Self(format!("covernode_{node_number:0>3}"))
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl FromStr for CoverNodeIdentity {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        CoverNodeIdentity::new(s)
    }
}

impl Deref for CoverNodeIdentity {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<String> for CoverNodeIdentity {
    fn as_ref(&self) -> &String {
        &self.0
    }
}

impl Display for CoverNodeIdentity {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<CoverNodeIdentity> for String {
    fn from(value: CoverNodeIdentity) -> Self {
        value.0
    }
}
