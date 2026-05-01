use std::{borrow::Cow, collections::HashMap};

use testcontainers::{core::WaitFor, Image};

use crate::secrets::{API_AWS_ACCESS_KEY_ID_SECRET, API_AWS_SECRET_ACCESS_KEY_SECRET};

const NAME: &str = "minio/minio";
const TAG: &str = "RELEASE.2025-09-07T16-13-09Z";

#[derive(Debug, Clone)]
pub struct MinioArgs {}

impl MinioArgs {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for MinioArgs {
    fn default() -> Self {
        Self::new()
    }
}

impl MinioArgs {
    pub fn into_cmd(self) -> Vec<String> {
        vec!["server".into(), "/data".into()]
    }
}

#[derive(Debug)]
pub struct Minio {
    env_vars: HashMap<String, String>,
}

impl Default for Minio {
    fn default() -> Self {
        let mut env_vars = HashMap::new();
        // set minio credentials to the API's aws credentials so that it can access minio
        env_vars.insert(
            "MINIO_ROOT_USER".to_owned(),
            API_AWS_ACCESS_KEY_ID_SECRET.into(),
        );
        env_vars.insert(
            "MINIO_ROOT_PASSWORD".to_owned(),
            API_AWS_SECRET_ACCESS_KEY_SECRET.into(),
        );

        Self { env_vars }
    }
}

impl Image for Minio {
    fn name(&self) -> &str {
        NAME
    }

    fn tag(&self) -> &str {
        TAG
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::message_on_stderr("MinIO Object Storage Server")]
    }

    fn env_vars(
        &self,
    ) -> impl IntoIterator<Item = (impl Into<Cow<'_, str>>, impl Into<Cow<'_, str>>)> {
        self.env_vars.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }
}
