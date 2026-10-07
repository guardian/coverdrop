use std::{borrow::Cow, collections::HashMap, env};

use testcontainers::{core::WaitFor, Image};

use crate::secrets::{API_AWS_ACCESS_KEY_ID_SECRET, API_AWS_SECRET_ACCESS_KEY_SECRET};

#[derive(Debug, Clone)]
pub struct GarageArgs {}

impl GarageArgs {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for GarageArgs {
    fn default() -> Self {
        Self::new()
    }
}

impl GarageArgs {
    pub fn into_cmd(self) -> Vec<String> {
        // `--single-node` configures a layout for a single-node cluster, and
        // `--default-bucket` creates the access key specified by the
        // `GARAGE_DEFAULT_*` env vars below so that the API can authenticate.
        vec![
            "server".into(),
            "--single-node".into(),
            "--default-bucket".into(),
        ]
    }
}

#[derive(Debug)]
pub struct Garage {
    name: String,
    tag: String,
    env_vars: HashMap<String, String>,
}

impl Default for Garage {
    fn default() -> Self {
        let mut env_vars = HashMap::new();
        // set garage's default access key to the API's aws credentials so that it can access garage
        env_vars.insert(
            "GARAGE_DEFAULT_ACCESS_KEY".to_owned(),
            API_AWS_ACCESS_KEY_ID_SECRET.into(),
        );
        env_vars.insert(
            "GARAGE_DEFAULT_SECRET_KEY".to_owned(),
            API_AWS_SECRET_ACCESS_KEY_SECRET.into(),
        );
        // Unused by the tests, but required alongside `--default-bucket`.
        env_vars.insert(
            "GARAGE_DEFAULT_BUCKET".to_owned(),
            "default-bucket".to_owned(),
        );

        Self {
            name: env::var("GARAGE_IMAGE_NAME").unwrap_or("test_coverdrop_garage".into()),
            tag: env::var("GARAGE_IMAGE_TAG").unwrap_or("dev".into()),
            env_vars,
        }
    }
}

impl Image for Garage {
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
