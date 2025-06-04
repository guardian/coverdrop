use std::collections::HashMap;

use testcontainers::{core::WaitFor, Image, ImageArgs};

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

impl ImageArgs for PostgresArgs {
    fn into_iterator(self) -> Box<dyn Iterator<Item = String>> {
        let command = "docker-entrypoint.sh -c 'track_commit_timestamp=on'".into();

        Box::new(vec!["/bin/bash".into(), "-c".into(), command].into_iter())
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
    type Args = PostgresArgs;

    fn name(&self) -> String {
        NAME.to_owned()
    }

    fn tag(&self) -> String {
        TAG.to_owned()
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::message_on_stdout(
            "database system is ready to accept connections",
        )]
    }

    fn env_vars(&self) -> Box<dyn Iterator<Item = (&String, &String)> + '_> {
        Box::new(self.env_vars.iter())
    }
}
