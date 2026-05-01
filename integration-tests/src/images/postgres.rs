use std::{borrow::Cow, collections::HashMap};

use testcontainers::{core::WaitFor, Image};

use crate::constants::{POSTGRES_DB, POSTGRES_PASSWORD, POSTGRES_USER};

const NAME: &str = "postgres";
const TAG: &str = "14.5";

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
    env_vars: HashMap<String, String>,
}

impl Default for Postgres {
    fn default() -> Self {
        let mut env_vars = HashMap::new();
        env_vars.insert("POSTGRES_USER".to_owned(), POSTGRES_USER.into());
        env_vars.insert("POSTGRES_PASSWORD".to_owned(), POSTGRES_PASSWORD.into());
        env_vars.insert("POSTGRES_DB".to_owned(), POSTGRES_DB.into());

        Self { env_vars }
    }
}

impl Image for Postgres {
    fn name(&self) -> &str {
        NAME
    }

    fn tag(&self) -> &str {
        TAG
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::message_on_stdout(
            "database system is ready to accept connections",
        )]
    }

    fn env_vars(
        &self,
    ) -> impl IntoIterator<Item = (impl Into<Cow<'_, str>>, impl Into<Cow<'_, str>>)> {
        self.env_vars.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }
}
