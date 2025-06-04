use crate::clap::Stage;

pub const MESSAGE_CANARY_NAMESPACE: &str = "CoverDrop/MessageCanary";
pub const COVERNODE_NAMESPACE: &str = "CoverDrop/CoverNode";
pub const API_NAMESPACE: &str = "CoverDrop/API";
pub const IDENTITY_API_NAMESPACE: &str = "CoverDrop/IdentityAPI";
pub const U2J_APPENDER_NAMESPACE: &str = "CoverDrop/U2JAppender";

pub async fn init_metrics(
    namespace: impl Into<String>,
    stage: &Stage,
) -> anyhow::Result<(), metrics_cloudwatch::Error> {
    let namespace = namespace.into();

    tracing::info!(
        "initializing metrics with namespace {} and stage {}",
        namespace,
        stage.as_guardian_str(),
    );

    // TODO if possible, build config rather than loading from env
    let config = aws_config::load_from_env().await;
    let cloudwatch_client = aws_sdk_cloudwatch::Client::new(&config);

    metrics_cloudwatch::Builder::new()
        .cloudwatch_namespace(namespace)
        .default_dimension("Stage", stage.as_guardian_str())
        .send_interval_secs(60)
        .send_timeout_secs(60)
        .init_thread(cloudwatch_client, metrics::set_global_recorder)
}
