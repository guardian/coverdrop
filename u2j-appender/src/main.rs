use axum::{
    extract::FromRef,
    routing::{get, post},
    Router,
};
use clap::Parser;
use common::{
    metrics::{init_metrics, U2J_APPENDER_NAMESPACE},
    tracing::init_tracing,
};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::net::TcpListener;
use u2j_appender::{
    cli::Cli,
    controllers::{get_healthcheck, post_u2j_message},
    kinesis_client::KinesisClient,
    DEFAULT_PORT,
};

#[derive(Clone, FromRef)]
pub struct U2JAppenderState {
    pub kinesis_client: KinesisClient,
}

impl U2JAppenderState {
    pub fn new(kinesis_client: KinesisClient) -> Self {
        Self { kinesis_client }
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let _ = start(cli).await.map_err(|e| {
        tracing::error!("{}", e);
    });
}

async fn start(cli: Cli) -> anyhow::Result<()> {
    init_tracing("info");
    init_metrics(U2J_APPENDER_NAMESPACE, &cli.stage).await?;

    tracing::info!("Cli args: {cli:?}");

    let kinesis_client = KinesisClient::new(
        cli.kinesis_endpoint,
        cli.kinesis_u2j_stream,
        &cli.aws_config,
    )
    .await;

    let u2j_appender_state = U2JAppenderState::new(kinesis_client);

    let app = Router::new()
        .route("/healthcheck", get(get_healthcheck))
        .route("/user/messages", post(post_u2j_message))
        .layer(axum_metrics::MetricLayer::default())
        .with_state(u2j_appender_state);

    let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), DEFAULT_PORT);

    tracing::info!("Starting server on http://{:?}", socket_addr);
    let listener = TcpListener::bind(&socket_addr).await?;

    axum::serve(listener, app).await?;

    Ok(())
}
