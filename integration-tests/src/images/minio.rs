use std::{borrow::Cow, collections::HashMap, env};

use testcontainers::{core::WaitFor, Image};

use crate::secrets::{API_AWS_ACCESS_KEY_ID_SECRET, API_AWS_SECRET_ACCESS_KEY_SECRET};

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
    name: String,
    tag: String,
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

        Self {
            name: env::var("MINIO_IMAGE_NAME").unwrap_or("test_coverdrop_minio".into()),
            tag: env::var("MINIO_IMAGE_TAG").unwrap_or("dev".into()),
            env_vars,
        }
    }
}

impl Image for Minio {
    fn name(&self) -> &str {
        &self.name
    }

    fn tag(&self) -> &str {
        &self.tag
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::healthcheck()]
    }

    fn env_vars(
        &self,
    ) -> impl IntoIterator<Item = (impl Into<Cow<'_, str>>, impl Into<Cow<'_, str>>)> {
        self.env_vars.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }
}
