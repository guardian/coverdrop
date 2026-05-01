use std::{borrow::Cow, collections::HashMap};

use testcontainers::{core::WaitFor, Image};

const NAME: &str = "varnish";
const TAG: &str = "6.0";

#[derive(Debug, Clone)]
pub struct VarnishArgs {}

impl VarnishArgs {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for VarnishArgs {
    fn default() -> Self {
        Self::new()
    }
}

impl VarnishArgs {
    pub fn into_cmd(self) -> Vec<String> {
        let command = "/usr/local/bin/docker-varnish-entrypoint".into();

        vec!["/bin/bash".into(), "-c".into(), command]
    }
}

#[derive(Debug)]
pub struct Varnish {
    env_vars: HashMap<String, String>,
}

impl Default for Varnish {
    fn default() -> Self {
        let mut env_vars = HashMap::new();
        env_vars.insert("VARNISH_SIZE".to_owned(), "2G".into());
        Self { env_vars }
    }
}

impl Image for Varnish {
    fn name(&self) -> &str {
        NAME
    }

    fn tag(&self) -> &str {
        TAG
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::message_on_stderr("said Child starts")]
    }

    fn env_vars(
        &self,
    ) -> impl IntoIterator<Item = (impl Into<Cow<'_, str>>, impl Into<Cow<'_, str>>)> {
        self.env_vars.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }
}
