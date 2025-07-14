use axum::extract::FromRef;
use common::api::api_client::ApiClient;
use identity_api_database::Database;

#[derive(Clone, FromRef)]
pub struct IdentityApiState {
    pub api_client: ApiClient,
    pub database: Database,
}

impl IdentityApiState {
    pub fn new(api_client: ApiClient, database: Database) -> Self {
        Self {
            api_client,
            database,
        }
    }
}
