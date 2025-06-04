use axum::{
    routing::{get, post},
    Extension, Router,
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

    let app = Router::new()
        .route("/healthcheck", get(get_healthcheck))
        .route("/user/messages", post(post_u2j_message))
        .layer(Extension(kinesis_client))
        .layer(axum_metrics::MetricLayer::default());

    let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), DEFAULT_PORT);

    tracing::info!("Starting server on http://{:?}", socket_addr);
    let listener = TcpListener::bind(&socket_addr).await?;

    axum::serve(listener, app).await?;

    Ok(())
}
