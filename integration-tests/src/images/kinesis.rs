use std::env;
use testcontainers::{core::WaitFor, Image};

#[derive(Debug)]
pub struct Kinesis {
    name: String,
    tag: String,
}

impl Default for Kinesis {
    fn default() -> Self {
        Self {
            name: env::var("KINESIS_IMAGE_NAME").unwrap_or("test_coverdrop_kinesis".into()),
            tag: env::var("KINESIS_IMAGE_TAG").unwrap_or("dev".into()),
        }
    }
}

impl Image for Kinesis {
    fn name(&self) -> &str {
        &self.name
    }

    fn tag(&self) -> &str {
        &self.tag
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::message_on_stdout("Listening at")]
    }
}
