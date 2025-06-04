use aws_config::{default_provider::credentials::DefaultCredentialsChain, BehaviorVersion};
use aws_sdk_ssm::{config::Region, types::ParameterType, Client, Config};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SsmError {
    #[error("Parameter not found")]
    ParameterNotFound,
    #[error("Parameter is empty")]
    ParameterEmptyValue,
}

#[derive(Clone)]
pub struct SsmClient {
    inner: Client,
}

impl SsmClient {
    async fn build_credentials(profile: Option<String>) -> DefaultCredentialsChain {
        let mut builder = DefaultCredentialsChain::builder();
        if let Some(profile) = profile {
            builder = builder.profile_name(&profile);
        }

        builder.build().await
    }

    pub async fn new(region: String, profile: Option<String>) -> SsmClient {
        let region = Region::new(region);
        let credentials_provider = SsmClient::build_credentials(profile).await;

        let config = Config::builder()
            .behavior_version_latest()
            .region(region)
            .credentials_provider(credentials_provider)
            .build();

        let inner = Client::from_conf(config);

        SsmClient { inner }
    }

    /// When run in AWS, configuration can be loaded from the environment
    /// making parameters like `region` and `profile` redundant
    pub async fn new_in_aws() -> SsmClient {
        let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
        let config = aws_sdk_ssm::config::Builder::from(&config).build();

        let inner = Client::from_conf(config);

        SsmClient { inner }
    }

    pub async fn get_parameter(&self, parameter: &str) -> anyhow::Result<String> {
        let parameter_output = self.inner.get_parameter().name(parameter).send().await?;

        let Some(parameter) = parameter_output.parameter else {
            Err(SsmError::ParameterNotFound)?
        };
        let Some(value) = parameter.value else {
            Err(SsmError::ParameterEmptyValue)?
        };

        Ok(value)
    }

    pub async fn get_all_parameter_versions(
        &self,
        parameter: &str,
        max_page_count: usize,
    ) -> anyhow::Result<Vec<String>> {
        let mut history = vec![];
        let mut count: usize = 0;
        let mut paginator = self
            .inner
            .get_parameter_history()
            .name(parameter)
            .into_paginator()
            .send();

        while let Some(page) = paginator.next().await {
            count += 1;

            if count > max_page_count {
                anyhow::bail!(
                    "Too many pages pulled from parameter store for {}",
                    parameter
                );
            } else {
                let page = page?;
                if let Some(parameters) = page.parameters {
                    let parameters = parameters
                        .iter()
                        .flat_map(|p| p.value().map(|v| v.to_owned()));
                    history.extend(parameters);
                }
            }
        }

        Ok(history)
    }

    pub async fn put_string_parameter(
        &self,
        name: impl Into<String>,
        value: impl Into<String>,
        description: impl Into<String>,
    ) -> anyhow::Result<()> {
        self.inner
            .put_parameter()
            .name(name)
            .value(value)
            .description(description)
            .r#type(ParameterType::String)
            .overwrite(true)
            .send()
            .await?;

        Ok(())
    }
}
