use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use axum::{routing::get, Extension, Router};
use common::tracing::TracingReloadHandle;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

use crate::{
    controllers::{general::get_healthcheck, public_keys::get_public_keys},
    key_state::KeyState,
};

pub async fn serve(
    port: u16,
    key_state: KeyState,
    tracing_reload_handle: TracingReloadHandle,
) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/healthcheck", get(get_healthcheck))
        .route("/public-keys", get(get_public_keys));

    let app = Router::new()
        .nest("/v1/", app)
        .layer(TraceLayer::new_for_http())
        .layer(Extension(key_state))
        .layer(Extension(tracing_reload_handle));

    let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port);

    tracing::info!("Starting server on http://{:?}", socket_addr);
    let listener = TcpListener::bind(&socket_addr).await?;

    axum::serve(listener, app).await?;

    Ok(())
}
