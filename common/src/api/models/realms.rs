use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, ValueEnum, Clone)]
#[serde(rename_all = "lowercase", deny_unknown_fields)]
pub enum Realm {
    User,
    Journalist,
}

impl From<&Realm> for &str {
    fn from(realm: &Realm) -> Self {
        match realm {
            Realm::User => "user",
            Realm::Journalist => "journalist",
        }
    }
}
