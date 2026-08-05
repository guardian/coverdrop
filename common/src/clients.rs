//! A module containing utility functions to handle HTTP responses from `reqwest`

use http::HeaderMap;
use reqwest::{Client, Response};
use serde::de::DeserializeOwned;
use std::time::Duration;

use crate::Error;

fn new_reqwest_client_with_defaults(
    timeout: Option<Duration>,
    default_headers: Option<HeaderMap>,
) -> Client {
    // We set the max number of allowed idle connections to 0 to avoid
    // a race condition where a connection is selected from the pool and
    // written to at the same time the server is closing it.
    // More details here:
    // https://github.com/hyperium/hyper/issues/2136#issuecomment-589345238
    Client::builder()
        .pool_max_idle_per_host(0)
        // A reqwest client has no default timeout so it's important we set one.
        .timeout(timeout.unwrap_or_else(|| Duration::from_secs(45)))
        .default_headers(default_headers.unwrap_or_else(HeaderMap::new))
        .build()
        .expect("Build reqwest client")
}

pub fn new_reqwest_client_with_timeout(timeout: Duration) -> Client {
    new_reqwest_client_with_defaults(Some(timeout), None)
}

pub fn new_reqwest_client_with_default_headers(default_headers: HeaderMap) -> Client {
    new_reqwest_client_with_defaults(None, Some(default_headers))
}

pub fn new_reqwest_client() -> Client {
    new_reqwest_client_with_defaults(None, None)
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

/// Turns the response to text and captures errors
pub async fn handle_response_text(resp: Response) -> anyhow::Result<String> {
    if resp.status().is_success() {
        let text = resp.text().await?;
        Ok(text)
    } else {
        handle_error(resp).await
    }
}
