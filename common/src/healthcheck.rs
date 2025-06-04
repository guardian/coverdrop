use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct HealthCheck {
    pub name: String,
    pub status: String,
    pub commit: Option<String>,
    pub branch: Option<String>,
}

impl HealthCheck {
    pub fn new(name: &str, status: &str) -> Self {
        let commit = option_env!("GIT_COMMIT").map(|s| s.to_string());
        let branch = option_env!("GIT_BRANCH").map(|s| s.to_string());

        Self {
            name: name.to_string(),
            status: status.to_string(),
            commit,
            branch,
        }
    }
}
