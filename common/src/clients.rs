//! A module containing utility functions to handle HTTP responses from `reqwest`

use reqwest::{Client, Response};
use serde::de::DeserializeOwned;

use crate::Error;

pub fn new_reqwest_client() -> Client {
    // We set the max number of allowed idle connections to 0 to avoid
    // a race condition where a connection is selected from the pool and
    // written to at the same time the server is closing it.
    // More details here:
    // https://github.com/hyperium/hyper/issues/2136#issuecomment-589345238
    Client::builder()
        .pool_max_idle_per_host(0)
        .build()
        .expect("Build reqwest client")
}

async fn handle_error<T>(resp: Response) -> anyhow::Result<T> {
    let status = resp.status();
    let error_text = resp.text().await?;
    tracing::error!("Error {}: {}", status, error_text);
    Err(Error::Api(status, error_text))?
}

/// Use this when you don't need to do anything with the response from the server (e.g. you don't need
/// to turn it to text or json) but just want to capture errors
pub async fn handle_response(resp: Response) -> anyhow::Result<()> {
    if resp.status().is_success() {
        Ok(())
    } else {
        handle_error(resp).await
    }
}

/// Turns the response to Json and captures errors
pub async fn handle_response_json<T>(resp: Response) -> anyhow::Result<T>
where
    T: DeserializeOwned,
{
    if resp.status().is_success() {
        let json = resp.json().await?;
        Ok(json)
    } else {
        handle_error(resp).await
    }
}
