use crate::{
    api::models::messages::{user_to_covernode_message::EncryptedUserToCoverNodeMessage, Message},
    clients::handle_response,
};
use reqwest::{Client, Response, Url};

#[derive(Clone, Debug)]
/// A thin wrapper around [`reqwest::Client`] to make HTTP requests
/// to the U2J appender
pub struct MessagingClient {
    inner: Client,
    base_url: Url,
}

impl MessagingClient {
    pub fn new(base_url: Url) -> Self {
        // We set the max number of allowed idle connections to 0 to avoid
        // a race condition where a connection is selected from the pool and
        // written to at the same time the server is closing it.
        // More details here:
        // https://github.com/hyperium/hyper/issues/2136#issuecomment-589345238
        let inner = Client::builder().pool_max_idle_per_host(0).build().unwrap();

        Self { inner, base_url }
    }

    pub async fn healthcheck(&self) -> anyhow::Result<Response> {
        let mut url = self.base_url.clone();
        url.path_segments_mut().unwrap().push("healthcheck");

        let healthcheck = self.inner.get(url).send().await?;

        Ok(healthcheck)
    }

    pub async fn post_user_message(
        &self,
        message: EncryptedUserToCoverNodeMessage,
    ) -> anyhow::Result<()> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .unwrap()
            .push("user")
            .push("messages");

        let resp = self
            .inner
            .post(url)
            .json(&Message { data: message })
            .send()
            .await?;

        handle_response(resp).await
    }
}
