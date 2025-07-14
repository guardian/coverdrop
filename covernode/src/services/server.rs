use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use axum::{extract::FromRef, routing::get, Router};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

use crate::{
    controllers::{general::get_healthcheck, public_keys::get_public_keys},
    key_state::KeyState,
};

#[derive(Clone, FromRef)]
pub struct CoverNodeState {
    pub key_state: KeyState,
}

impl CoverNodeState {
    pub fn new(key_state: KeyState) -> Self {
        CoverNodeState { key_state }
    }
}

pub async fn serve(port: u16, key_state: KeyState) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/healthcheck", get(get_healthcheck))
        .route("/public-keys", get(get_public_keys));

    let covernode_state = CoverNodeState::new(key_state);

    let app = Router::new()
        .nest("/v1/", app)
        .layer(TraceLayer::new_for_http())
        .with_state(covernode_state);

    let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port);

    tracing::info!("Starting server on http://{:?}", socket_addr);
    let listener = TcpListener::bind(&socket_addr).await?;

    axum::serve(listener, app).await?;

    Ok(())
}
