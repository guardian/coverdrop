use axum::{
    routing::{get, post},
    Router,
};
use chrono::Duration;
use clap::Parser;
use common::{
    api::api_client::ApiClient,
    metrics::{init_metrics, IDENTITY_API_NAMESPACE},
    task::{HeartbeatTask, Task as _, TaskRunner},
    time,
    tracing::{init_tracing, log_task_exit, log_task_result_exit},
};
use identity_api::{
    cli::Cli,
    controllers::{
        general::get_healthcheck, post_keys::post_rotate_covernode_id_key,
        public_keys::get_public_keys,
    },
    identity_api_state::IdentityApiState,
    tasks::{CheckFileSystemForKeysTask, DeleteExpiredKeysTask, RotateJournalistIdPublicKeysTask},
    DEFAULT_PORT,
};
use identity_api_database::Database;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let _ = start(cli).await.map_err(|e| {
        tracing::error!("{}", e);
    });
}

async fn start(cli: Cli) -> anyhow::Result<()> {
    init_tracing("info");
    init_metrics(IDENTITY_API_NAMESPACE, &cli.stage).await?;

    tracing::info!("Cli args: {cli:?}");

    let api_client = ApiClient::new(cli.api_url);

    let database = Database::open(&cli.db_path, &cli.db_password).await?;

    // Load trust anchors and insert them into the database on startup
    let now = time::now();
    let trust_anchors = trust_anchors::get_trust_anchors(&cli.stage, now)?;
    tracing::info!(
        "Inserting {} trust anchors into database",
        trust_anchors.len()
    );
    for anchor_org_pk in &trust_anchors {
        database
            .insert_anchor_organization_pk(anchor_org_pk, now)
            .await?;
    }

    let check_file_system_for_keys_task =
        CheckFileSystemForKeysTask::new(Duration::seconds(60), cli.keys_path, database.clone());

    // Make sure any keys in the file system are loaded before doing anything else
    check_file_system_for_keys_task.run().await?;

    let rotate_journalist_id_pk_task = RotateJournalistIdPublicKeysTask::new(
        Duration::seconds(15),
        api_client.clone(),
        database.clone(),
    );

    let delete_expired_keys_task =
        DeleteExpiredKeysTask::new(Duration::seconds(60), database.clone());

    let heartbeat_task = HeartbeatTask::default();

    let mut cron_tasks = tokio::task::spawn({
        let mut runner = TaskRunner::new(cli.task_runner_mode);

        runner.add_task(check_file_system_for_keys_task).await;
        runner.add_task(rotate_journalist_id_pk_task).await;
        runner.add_task(delete_expired_keys_task).await;
        runner.add_task(heartbeat_task).await;

        async move { runner.run().await }
    });

    let mut web_server = tokio::task::spawn(async move {
        let identity_api_state = IdentityApiState::new(api_client, database);

        let app = Router::new()
            // General
            .route("/healthcheck", get(get_healthcheck))
            .route("/public-keys", get(get_public_keys))
            // CoverNode key rotation
            .route(
                "/public-keys/covernode/me/rotate-id-key",
                post(post_rotate_covernode_id_key),
            )
            .with_state(identity_api_state);

        let app = Router::new()
            .nest("/v1/", app)
            .layer(TraceLayer::new_for_http());

        let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), DEFAULT_PORT);

        tracing::info!("Starting identity API server on http://{:?}", socket_addr);
        let listener = TcpListener::bind(&socket_addr).await?;

        axum::serve(listener, app).await
    });

    tokio::select! {
        r = (&mut cron_tasks) => {
            log_task_exit("journalist id rotation service", r);

            web_server.abort();
        },
        r = (&mut web_server) => {
            log_task_result_exit("web server", r);

            cron_tasks.abort();
        },
    }

    Ok(())
}
