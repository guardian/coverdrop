use std::{marker::PhantomData, time::Duration};

use reqwest::Url;

use crate::{clients::new_reqwest_client, service::CoverDropService};

pub struct TaskApiClient<T: CoverDropService> {
    client: reqwest::Client,
    base_url: Url,
    marker: PhantomData<T>,
}

impl<T: CoverDropService> TaskApiClient<T> {
    pub fn new(base_url: Url) -> anyhow::Result<Self> {
        let client = new_reqwest_client();
        Ok(Self {
            client,
            base_url,
            marker: PhantomData,
        })
    }

    pub async fn trigger_task(&self, task_name: &str) -> anyhow::Result<()> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .map_err(|_| anyhow::anyhow!("Failed to set path segments"))?
            .push("tasks")
            .push(task_name)
            .push("trigger");

        let result = self
            .client
            .post(url)
            .timeout(Duration::from_secs(30))
            .send()
            .await?;

        if result.status().is_success() {
            Ok(())
        } else {
            anyhow::bail!(
                "Non-success response from triggered task: {}",
                result.status()
            );
        }
    }
}
