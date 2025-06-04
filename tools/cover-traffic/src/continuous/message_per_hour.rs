use common::aws::ssm::{client::SsmClient, ssm_value::SsmValue};

// Transparently get the parameters for continuous mode cover traffic generation
pub enum MessagesPerHour {
    ParameterStore(SsmValue<u32>),
    Manual(u32),
}

impl MessagesPerHour {
    pub async fn new_for_parameter_store(
        ssm_client: &SsmClient,
        parameter: &str,
    ) -> anyhow::Result<Self> {
        let v = SsmValue::<u32>::new(ssm_client, parameter).await?;

        Ok(Self::ParameterStore(v))
    }

    pub fn new_for_manual(v: u32) -> Self {
        Self::Manual(v)
    }

    pub fn value(&self) -> &u32 {
        match self {
            MessagesPerHour::ParameterStore(v) => v.value(),
            MessagesPerHour::Manual(v) => v,
        }
    }

    pub async fn update(&mut self) -> anyhow::Result<()> {
        match self {
            MessagesPerHour::ParameterStore(v) => {
                v.update().await?;
                Ok(())
            }
            _ => Ok(()),
        }
    }
}
