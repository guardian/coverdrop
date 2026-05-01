use axum::routing::{get, post};
use axum::Router;
use clap::Parser;
use common::api::api_client::ApiClient;
use common::clap::Stage;
use common::metrics::{init_metrics, DELIVERY_SERVICE_NAMESPACE};
use common::time;
use common::tracing::init_tracing;
use delivery_service::app_state::AppState;
use delivery_service::cli::Cli;
use delivery_service::controllers::clients::{
    consume_key_package, get_clients, publish_key_packages, register_client,
};
use delivery_service::controllers::general::get_healthcheck;
use delivery_service::controllers::messages::{add_members, receive_messages, send_message};
use delivery_service::helpers::fetch_and_parse_db_url_secret;
use delivery_service::services::database::Database;
use delivery_service::DEFAULT_PORT;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let _ = start(cli).await.map_err(|e| {
        tracing::error!("Fatal error in delivery service: {}", e);
    });
}

async fn start(cli: Cli) -> anyhow::Result<()> {
    init_metrics(DELIVERY_SERVICE_NAMESPACE, &cli.stage).await?;

    //
    // Initialize tracing
    //
    if cli.stage == Stage::Development {
        init_tracing("info");
    } else {
        // required to enable CloudWatch error logging by the runtime
        lambda_http::tracing::init_default_subscriber();
    }

    tracing::info!("Cli args: {cli:?}");

    //
    // Set up services
    //

    let db_url: &String = match (&cli.db_url, &cli.db_secret_arn) {
        (Some(db_url), _) => {
            tracing::info!("Using database URL from COVERDROP_DELIVERY_SERVICE_DB_URL");
            db_url
        }
        (None, Some(secret_arn)) => {
            tracing::info!("Resolving database URL from secret ARN");
            &fetch_and_parse_db_url_secret(secret_arn, "coverdrop_delivery_service_db").await?
        }
        (None, None) => {
            anyhow::bail!(
                "Either COVERDROP_DELIVERY_SERVICE_DB_URL or COVERDROP_DELIVERY_SERVICE_DB_SECRET_ARN must be set"
            );
        }
    };

    let db = Database::new(db_url).await?;

    tracing::info!("Database initialization complete");

    let api_client = ApiClient::new(cli.api_url);

    let now = time::now();
    let trust_anchors = trust_anchors::get_trust_anchors(&cli.stage, now)?;
    tracing::info!("Loaded {} trust anchors", trust_anchors.len());

    let app_state = AppState::new(db, api_client, trust_anchors);

    tracing::info!("Building router...");
    let app = Router::new()
        .route("/healthcheck", get(get_healthcheck))
        .route("/clients/register", post(register_client))
        .route("/clients/list", post(get_clients))
        .route("/clients/key_package/publish", post(publish_key_packages))
        .route("/clients/key_package/consume", post(consume_key_package))
        .route("/group/add_members", post(add_members))
        .route("/send/message", post(send_message))
        .route("/receive", post(receive_messages))
        .with_state(app_state);

    let app = Router::new()
        .nest("/v1/", app)
        .layer(TraceLayer::new_for_http())
        .layer(axum_metrics::MetricLayer::default());

    tracing::info!("Router built successfully");

    if cli.stage == Stage::Development {
        tracing::info!("In DEV mode - running without lambda runtime");
        let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), DEFAULT_PORT);

        tracing::info!("Starting server on http://{:?}", socket_addr);
        let listener = TcpListener::bind(&socket_addr).await?;

        axum::serve(listener, app).await?;
    } else {
        tracing::info!("Starting Lambda server");
        lambda_http::run(app)
            .await
            .map_err(|e| anyhow::anyhow!("Lambda runtime error: {}", e))?;
    }

    Ok(())
}
