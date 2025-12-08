use std::time::Duration;

use aws_config::default_provider::credentials::DefaultCredentialsChain;
use aws_sdk_s3::config::Region;
use aws_sdk_s3::operation::get_object::GetObjectOutput;
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::types::Object;
use aws_sdk_s3::Client;
use reqwest::Url;

use crate::clap::AwsConfig;

#[derive(Clone)]
pub struct S3Client {
    inner: Client,
}

impl S3Client {
    async fn build_credentials(profile: Option<String>) -> DefaultCredentialsChain {
        let mut builder = DefaultCredentialsChain::builder();
        if let Some(profile) = profile {
            builder = builder.profile_name(&profile);
        }

        builder.build().await
    }

    pub async fn new(aws_config: AwsConfig, s3_endpoint_url: Url) -> S3Client {
        let region = Region::new(aws_config.region);
        let credentials_provider = S3Client::build_credentials(aws_config.profile).await;

        let config = aws_sdk_s3::Config::builder()
            .behavior_version_latest()
            .region(region)
            .force_path_style(true) // needed for minio
            .credentials_provider(credentials_provider)
            .endpoint_url(s3_endpoint_url.as_str())
            .build();

        let inner = Client::from_conf(config);

        S3Client { inner }
    }

    pub async fn create_presigned_put_object_url(
        &self,
        bucket: &str,
        key: &str,
        expires_in_seconds: u64,
    ) -> anyhow::Result<String> {
        let presigning_config = PresigningConfig::builder()
            .expires_in(Duration::from_secs(expires_in_seconds))
            .build()?;

        let request = self
            .inner
            .put_object()
            .bucket(bucket)
            .key(key)
            .presigned(presigning_config)
            .await?;

        Ok(request.uri().to_string())
    }

    #[cfg(feature = "integration-tests")]
    pub async fn create_bucket(&self, bucket: &str) -> anyhow::Result<()> {
        self.inner.create_bucket().bucket(bucket).send().await?;

        Ok(())
    }

    pub async fn list_objects(&self, bucket: &str, prefix: &str) -> anyhow::Result<Vec<Object>> {
        let resp = self
            .inner
            .list_objects_v2()
            .bucket(bucket)
            .prefix(prefix)
            .send()
            .await?;

        let keys: Vec<Object> = resp.contents.unwrap_or_default().to_vec();

        Ok(keys)
    }

    pub async fn get_object(&self, bucket: &str, key: &str) -> anyhow::Result<GetObjectOutput> {
        let object_output = self
            .inner
            .get_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await?;

        Ok(object_output)
    }
}
