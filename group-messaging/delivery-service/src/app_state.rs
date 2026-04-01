use crate::services::database::Database;
use axum::extract::FromRef;
use common::api::api_client::ApiClient;
use common::protocol::keys::AnchorOrganizationPublicKey;
use std::sync::Arc;

#[derive(Clone, FromRef)]
pub struct AppState {
    pub db: Database,
    pub api_client: ApiClient,
    pub trust_anchors: Arc<Vec<AnchorOrganizationPublicKey>>,
}

impl AppState {
    pub fn new(
        db: Database,
        api_client: ApiClient,
        trust_anchors: Vec<AnchorOrganizationPublicKey>,
    ) -> Self {
        AppState {
            db,
            api_client,
            trust_anchors: Arc::new(trust_anchors),
        }
    }
}
