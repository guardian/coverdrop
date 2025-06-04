use std::str::FromStr;

use super::client::SsmClient;

/// Updatable value from SSM parameter store
pub struct SsmValue<T: FromStr> {
    // SsmClient is a lightweight wrapper around a handle around the underlying SSM client
    // so it's not a big problem to have a client per SSM value since we generally don't expect
    // there to be a huge number of SsmValues.
    ssm_client: SsmClient,
    parameter: String,
    value: T,
}

impl<T: FromStr> SsmValue<T> {
    async fn fetch_value(parameter: &str, ssm_client: &SsmClient) -> anyhow::Result<T> {
        let str_value = ssm_client.get_parameter(parameter).await?;

        let Ok(value) = T::from_str(&str_value) else {
            anyhow::bail!(
                "Failed to parse value from SSM parameter store: {:?}",
                str_value
            )
        };

        Ok(value)
    }

    pub async fn new(ssm_client: &SsmClient, parameter: &str) -> anyhow::Result<Self> {
        let value = Self::fetch_value(parameter, ssm_client).await?;

        Ok(Self {
            ssm_client: ssm_client.clone(),
            parameter: parameter.to_string(),
            value,
        })
    }

    /// Get the latest version of this SSM parameter
    pub async fn update(&mut self) -> anyhow::Result<()> {
        self.value = Self::fetch_value(&self.parameter, &self.ssm_client).await?;
        Ok(())
    }

    pub fn value(&self) -> &T {
        &self.value
    }
}
