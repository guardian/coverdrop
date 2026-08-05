use std::{borrow::Cow, collections::HashMap, env};

use testcontainers::{core::WaitFor, Image};

use crate::constants::{POSTGRES_DB, POSTGRES_PASSWORD, POSTGRES_USER};

#[derive(Debug, Clone)]
pub struct PostgresArgs {}

impl PostgresArgs {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for PostgresArgs {
    fn default() -> Self {
        Self::new()
    }
}

impl PostgresArgs {
    pub fn into_cmd(self) -> Vec<String> {
        let command = "docker-entrypoint.sh -c 'track_commit_timestamp=on'".into();

        vec!["/bin/bash".into(), "-c".into(), command]
    }
}

#[derive(Debug)]
pub struct Postgres {
    name: String,
    tag: String,
    env_vars: HashMap<String, String>,
}

impl Default for Postgres {
    fn default() -> Self {
        let mut env_vars = HashMap::new();
        env_vars.insert("POSTGRES_USER".to_owned(), POSTGRES_USER.into());
        env_vars.insert("POSTGRES_PASSWORD".to_owned(), POSTGRES_PASSWORD.into());
        env_vars.insert("POSTGRES_DB".to_owned(), POSTGRES_DB.into());

        Self {
            name: env::var("POSTGRES_IMAGE_NAME").unwrap_or("test_coverdrop_postgres".into()),
            tag: env::var("POSTGRES_IMAGE_TAG").unwrap_or("dev".into()),
            env_vars,
        }
    }
}

impl Image for Postgres {
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
