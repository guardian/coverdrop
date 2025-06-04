use aws_config::profile::ProfileFileCredentialsProvider;
use aws_config::{Region, SdkConfig};
use common::clap::AwsConfig;

pub async fn get_sdk_config(aws_config: AwsConfig) -> SdkConfig {
    let credentials_provider = match aws_config.profile {
        Some(profile) => ProfileFileCredentialsProvider::builder()
            .profile_name(profile)
            .build(),
        None => {
            println!("No profile provided, using default credentials");
            ProfileFileCredentialsProvider::builder().build()
        }
    };

    aws_config::from_env()
        .region(Region::new(aws_config.region))
        .credentials_provider(credentials_provider)
        .load()
        .await
}
