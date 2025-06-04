use std::collections::HashMap;

use testcontainers::{core::WaitFor, Image, ImageArgs};

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

impl ImageArgs for VarnishArgs {
    fn into_iterator(self) -> Box<dyn Iterator<Item = String>> {
        let command = "/usr/local/bin/docker-varnish-entrypoint".into();

        Box::new(vec!["/bin/bash".into(), "-c".into(), command].into_iter())
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
    type Args = VarnishArgs;

    fn name(&self) -> String {
        NAME.to_owned()
    }

    fn tag(&self) -> String {
        TAG.to_owned()
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::message_on_stderr("said Child starts")]
    }

    fn env_vars(&self) -> Box<dyn Iterator<Item = (&String, &String)> + '_> {
        Box::new(self.env_vars.iter())
    }
}
