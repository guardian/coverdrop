use std::env;
use testcontainers::{core::WaitFor, Image};

#[derive(Debug, Default)]
pub struct Kinesis {}

impl Image for Kinesis {
    type Args = ();

    fn name(&self) -> String {
        env::var("KINESIS_IMAGE_NAME").unwrap_or("test_coverdrop_kinesis".into())
    }

    fn tag(&self) -> String {
        env::var("KINESIS_IMAGE_TAG").unwrap_or("dev".into())
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::message_on_stdout("Listening at")]
    }
}
