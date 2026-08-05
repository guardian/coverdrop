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
    #[ts(as = "String")]
    // delivery_service_url is optional because the delivery service isn't available in dev / multipass environments.
    pub delivery_service_url: Option<Url>,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Profiles(HashMap<String, Profile>);

impl Profiles {
    pub fn api_url(&self, profile_name: &str) -> Option<&Url> {
        self.0.get(profile_name).map(|p| &p.api_url)
    }

    pub fn delivery_service_url(&self, profile_name: &str) -> Option<&Url> {
        self.0
            .get(profile_name)
            .and_then(|p| p.delivery_service_url.as_ref())
    }
}
