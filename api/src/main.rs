use api::anchor_org_pk_cache::AnchorOrganizationPublicKeyCache;
use api::api_state::ApiState;
use api::cli::Cli;
use api::controllers::dead_drops::{
    get_journalist_dead_drops, get_journalist_recent_dead_drop_summary, get_user_dead_drops,
    get_user_recent_dead_drop_summary, post_journalist_dead_drops, post_user_dead_drops,
};
use api::controllers::general::{
    get_healthcheck, get_latest_status, post_reload_tracing, post_status_event,
};
use api::controllers::journalist_message::post_forward_journalist_to_covernode_msg;
use api::controllers::keys::{
    delete_journalist, get_journalist_id_pk_rotation_forms, get_journalist_id_pk_with_epoch,
    get_public_keys, patch_journalist, post_admin_key, post_covernode_id_key,
    post_covernode_msg_key, post_covernode_provisioning_key, post_journalist,
    post_journalist_id_key, post_journalist_id_pk_rotation_form, post_journalist_msg_key,
    post_journalist_provisioning_key,
};
use api::dead_drop_limits::DeadDropLimits;
use api::services::database::Database;
use api::services::tasks::{AnchorOrganizatioPublicKeyPollTask, DeleteOldDeadDropsTask};
use api::DEFAULT_PORT;
use axum::routing::{delete, get, patch, post};
use axum::Router;
use chrono::Duration;
use clap::Parser;
use common::aws::kinesis::client::KinesisClient;
use common::metrics::{init_metrics, API_NAMESPACE};
use common::task::TaskRunner;
use common::tracing::init_tracing_with_reload_handle;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

fn polling_seconds_to_duration(maybe_seconds: Option<i64>, default: Duration) -> Duration {
    maybe_seconds
        .map(|seconds| {
            if seconds <= 1 {
                panic!("Cannot poll more frequently than once a second");
            }

            Duration::seconds(seconds)
        })
        .unwrap_or(default)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    //
    // Parse command line args
    //
    let cli = Cli::parse();

    //
    // Initialize metrics
    //
    init_metrics(API_NAMESPACE, &cli.stage).await?;

    //
    // Initialize tracing
    //
    let tracing_reload_handle = init_tracing_with_reload_handle("info");
    tracing::info!("Cli args: {cli:?}");

    let delete_old_dead_drops_polling_period = polling_seconds_to_duration(
        cli.delete_old_dead_drops_polling_period_seconds,
        Duration::minutes(30),
    );

    let anchor_organisation_public_key_polling_period = polling_seconds_to_duration(
        cli.anchor_organization_public_key_polling_period_seconds,
        Duration::minutes(1),
    );

    //
    // Set up services
    //

    let db = Database::new(&cli.db_url, &cli.max_db_connections).await?;

    let kinesis_client = KinesisClient::new(
        &cli.kinesis_config,
        &cli.aws_config,
        vec![cli.kinesis_config.journalist_stream.clone()],
    )
    .await;

    //
    // Track the current trusted org pks in memory
    //
    let anchor_org_pks = AnchorOrganizationPublicKeyCache::default();

    tracing::debug!("Setting up background tasks");
    tokio::spawn({
        let delete_old_dead_drops_task =
            DeleteOldDeadDropsTask::new(delete_old_dead_drops_polling_period, db.clone());
        let anchor_org_pk_poll_task = AnchorOrganizatioPublicKeyPollTask::new(
            anchor_organisation_public_key_polling_period,
            cli.key_location,
            anchor_org_pks.clone(),
            db.clone(),
        );

        let mut runner = TaskRunner::new(cli.task_runner_mode);
        runner.add_task(delete_old_dead_drops_task).await;
        runner.add_task(anchor_org_pk_poll_task).await;

        async move {
            runner.run().await;
        }
    });

    //
    // Start web service
    //

    let dead_drop_limits = DeadDropLimits::new(
        cli.j2u_dead_drops_per_request_limit,
        cli.u2j_dead_drops_per_request_limit,
    );

    let api_state = ApiState::new(
        anchor_org_pks,
        db,
        kinesis_client,
        cli.default_journalist_id,
        tracing_reload_handle,
        dead_drop_limits,
    );

    let app = Router::new()
        // General
        .route("/healthcheck", get(get_healthcheck))
        .route("/status", get(get_latest_status).post(post_status_event))
        .route("/status/public-key", post(post_admin_key))
        .route("/logging", post(post_reload_tracing))
        // Public key infrastructure
        .route("/public-keys", get(get_public_keys))
        .route("/public-keys/journalists", post(post_journalist))
        .route("/public-keys/journalists/delete", delete(delete_journalist))
        .route(
            "/public-keys/journalists/update-profile",
            patch(patch_journalist),
        )
        .route(
            "/public-keys/covernode/provisioning-public-key",
            post(post_covernode_provisioning_key),
        )
        .route(
            "/public-keys/covernode/identity-public-key",
            post(post_covernode_id_key),
        )
        .route(
            "/public-keys/covernode/messaging-public-key",
            post(post_covernode_msg_key),
        )
        .route(
            "/public-keys/journalists/provisioning-public-key",
            post(post_journalist_provisioning_key),
        )
        .route(
            "/public-keys/journalists/identity-public-key-form",
            get(get_journalist_id_pk_rotation_forms).post(post_journalist_id_pk_rotation_form),
        )
        .route(
            "/public-keys/journalists/identity-public-key",
            post(post_journalist_id_key),
        )
        .route(
            "/public-keys/journalists/identity-public-key/{pk_hex}",
            get(get_journalist_id_pk_with_epoch),
        )
        .route(
            "/public-keys/journalists/messaging-public-key",
            post(post_journalist_msg_key),
        )
        // Dead drops
        .route(
            "/user/dead-drops",
            get(get_user_dead_drops).post(post_user_dead_drops),
        )
        .route(
            "/user/dead-drops/recent-summary",
            get(get_user_recent_dead_drop_summary),
        )
        .route(
            "/journalist/dead-drops",
            get(get_journalist_dead_drops).post(post_journalist_dead_drops),
        )
        .route(
            "/journalist/dead-drops/recent-summary",
            get(get_journalist_recent_dead_drop_summary),
        )
        .route(
            "/journalist-messages",
            post(post_forward_journalist_to_covernode_msg),
        )
        .with_state(api_state);

    let app = Router::new()
        .nest("/v1/", app)
        .layer(TraceLayer::new_for_http())
        .layer(axum_metrics::MetricLayer::default());

    let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), DEFAULT_PORT);

    tracing::info!("Starting server on http://{:?}", socket_addr);
    let listener = TcpListener::bind(&socket_addr).await?;

    axum::serve(listener, app).await?;

    Ok(())
}
