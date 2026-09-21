use std::time::Duration;

use testcontainers::core::client::ClientError;
use testcontainers::core::error::TestcontainersError;
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, ContainerRequest, Image};
use tokio::time::sleep;

const MAX_ATTEMPTS: u32 = 3;
const RETRY_DELAY: Duration = Duration::from_secs(5);

/// Pulling an image from the registry intermittently fails in CI with transient network
/// errors, e.g. `IOError { err: Custom { kind: Other, error: "bytes remaining on stream" } }`,
/// when the pull stream is severed mid-transfer. `testcontainers` has no built-in retry, so
/// we retry the whole start here.
///
/// Takes a closure rather than a [`ContainerRequest`] because the request is consumed by
/// [`AsyncRunner::start`] and has to be rebuilt for each attempt.
pub async fn start_with_retry<I, F>(
    container_name: &str,
    build_request: F,
) -> Result<ContainerAsync<I>, TestcontainersError>
where
    I: Image,
    F: Fn() -> ContainerRequest<I>,
{
    let mut attempt = 1;

    loop {
        match build_request().start().await {
            Ok(container) => return Ok(container),
            Err(e) if attempt < MAX_ATTEMPTS && is_transient_pull_error(&e) => {
                eprintln!(
                    "Attempt {attempt}/{MAX_ATTEMPTS} to start {container_name} container failed, retrying in {}s: {e}",
                    RETRY_DELAY.as_secs()
                );
                sleep(RETRY_DELAY).await;
                attempt += 1;
            }
            Err(e) => return Err(e),
        }
    }
}

fn is_transient_pull_error(error: &TestcontainersError) -> bool {
    matches!(
        error,
        TestcontainersError::Client(ClientError::PullImage { .. })
    )
}
