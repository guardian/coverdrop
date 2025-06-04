use std::collections::HashMap;

use reqwest::Url;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Profile {
    #[ts(as = "String")]
    pub api_url: Url,
}

#[derive(Debug, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Profiles(HashMap<String, Profile>);

impl Profiles {
    pub fn insert(&mut self, stage: impl Into<String>, url: Url) {
        self.0.insert(stage.into(), Profile { api_url: url });
    }

    pub fn api_url(&self, profile_name: &str) -> Option<&Url> {
        self.0.get(profile_name).map(|p| &p.api_url)
    }
}
