use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use axum::{routing::get, Router};
use canary_state::CanaryState;
use clap::Parser;
use cli::Cli;
use common::{
    api::api_client::ApiClient,
    metrics::{init_metrics, MESSAGE_CANARY_NAMESPACE},
    tracing::{init_tracing, log_task_result_exit},
    u2j_appender::messaging_client::MessagingClient,
};
use controllers::general::get_healthcheck;
use message_canary_database::database::Database;
use services::{
    create_undelivered_message_metrics, receive_j2u, receive_u2j, rotate_journalist_keys, send_j2u,
    send_u2j,
};
use tokio::net::TcpListener;

mod canary_state;
mod cli;
mod controllers;
mod services;

const DEFAULT_PORT: u16 = 3050;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing("debug");

    let cli = Cli::parse();

    tracing::info!("Cli args: {:?}", cli);

    init_metrics(MESSAGE_CANARY_NAMESPACE, &cli.stage).await?;

    let api_client = ApiClient::new(cli.api_url);
    let messaging_client = MessagingClient::new(cli.messaging_url);

    let db = Database::new(cli.db_url.as_ref()).await?;

    let canary_state = CanaryState::new(
        cli.keys_path,
        cli.vaults_path,
        api_client,
        messaging_client,
        db,
        cli.num_users,
    )
    .await?;

    //
    // Send
    //

    let mut send_u2j_service = tokio::spawn({
        let canary_state = canary_state.clone();
        async move { send_u2j(canary_state, cli.mph_u2j).await }
    });

    let mut send_j2u_service = tokio::spawn({
        let canary_state = canary_state.clone();
        async move { send_j2u(canary_state, cli.mph_j2u).await }
    });

    //
    // Receive
    //

    let mut receive_u2j_service = tokio::spawn({
        let canary_state = canary_state.clone();
        async move { receive_u2j(canary_state).await }
    });

    let mut receive_j2u_service = tokio::spawn({
        let canary_state = canary_state.clone();
        async move { receive_j2u(canary_state).await }
    });

    //
    // Journalist key rotationn
    //

    let mut rotate_journalist_keys_service = tokio::spawn({
        let canary_state = canary_state.clone();
        async move { rotate_journalist_keys(canary_state).await }
    });

    //
    // Metrics
    //

    let mut metrics_and_alerts_service = tokio::spawn({
        let canary_state = canary_state.clone();
        async move {
            create_undelivered_message_metrics(canary_state, cli.max_delivery_time_hours).await
        }
    });

    //
    // Web server
    //

    let mut web_server = tokio::task::spawn(async move {
        let app = Router::new()
            // General
            .route("/healthcheck", get(get_healthcheck));
        let app = Router::new().nest("/v1/", app);

        let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), DEFAULT_PORT);

        tracing::info!(
            "Starting Message Canary API server on http://{:?}",
            socket_addr
        );
        let listener = TcpListener::bind(&socket_addr).await?;

        axum::serve(listener, app).await
    });

    tokio::select! {
        r = (&mut send_u2j_service) => {
            log_task_result_exit("send U2J message service", r);

            send_j2u_service.abort();
            receive_u2j_service.abort();
            receive_j2u_service.abort();
            rotate_journalist_keys_service.abort();
            metrics_and_alerts_service.abort();
            web_server.abort();
        },
        r = (&mut send_j2u_service) => {
            log_task_result_exit("send J2U messages service", r);

            send_u2j_service.abort();
            receive_u2j_service.abort();
            receive_j2u_service.abort();
            rotate_journalist_keys_service.abort();
            metrics_and_alerts_service.abort();
            web_server.abort();
        },
        r = (&mut receive_u2j_service) => {
            log_task_result_exit("receive U2J message service", r);

            send_u2j_service.abort();
            send_j2u_service.abort();
            receive_j2u_service.abort();
            rotate_journalist_keys_service.abort();
            metrics_and_alerts_service.abort();
            web_server.abort();
        },
        r = (&mut receive_j2u_service) => {
            log_task_result_exit("receive J2U messages service", r);

            send_u2j_service.abort();
            send_j2u_service.abort();
            receive_u2j_service.abort();
            rotate_journalist_keys_service.abort();
            metrics_and_alerts_service.abort();
            web_server.abort();
        },
        r = (&mut rotate_journalist_keys_service) => {
            log_task_result_exit("rotate journalist keys service", r);

            send_u2j_service.abort();
            send_j2u_service.abort();
            receive_u2j_service.abort();
            receive_j2u_service.abort();
            metrics_and_alerts_service.abort();
            web_server.abort();
        },
        r = (&mut metrics_and_alerts_service) => {
            log_task_result_exit("metrics and alerts service", r);

            send_u2j_service.abort();
            send_j2u_service.abort();
            receive_u2j_service.abort();
            receive_j2u_service.abort();
            rotate_journalist_keys_service.abort();
            web_server.abort();
        },
        r = (&mut web_server) => {
            log_task_result_exit("web server service", r);

            send_u2j_service.abort();
            send_j2u_service.abort();
            receive_u2j_service.abort();
            receive_j2u_service.abort();
            rotate_journalist_keys_service.abort();
            metrics_and_alerts_service.abort();
        },
    }

    Ok(())
}
