use reqwest::Url;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    ConnectOptions,
};

use super::queries::{
    covernode_key_queries::CoverNodeKeyQueries, dead_drop_queries::DeadDropQueries,
    hierarchy_queries::HierarchyQueries, journalist_queries::JournalistQueries,
    organization_key_queries::OrganizationKeyQueries, system_key_queries::SystemKeyQueries,
    system_queries::SystemQueries,
};

#[derive(Clone)]
pub struct Database {
    pub hierarchy_queries: HierarchyQueries,
    pub organization_key_queries: OrganizationKeyQueries,
    pub covernode_key_queries: CoverNodeKeyQueries,
    pub dead_drop_queries: DeadDropQueries,
    pub journalist_queries: JournalistQueries,
    pub system_key_queries: SystemKeyQueries,
    pub system_queries: SystemQueries,
}

impl Database {
    pub async fn new(db_url: &str, max_connections: &Option<u32>) -> anyhow::Result<Database> {
        let url = Url::parse(db_url).expect("Parse db url");
        // We disable statement logging so no connection secrets are sent to logs
        let connect_options = PgConnectOptions::from_url(&url)?.disable_statement_logging();
        let pool = PgPoolOptions::new()
            .max_connections(max_connections.unwrap_or(10))
            .connect_with(connect_options)
            .await?;

        sqlx::migrate!().run(&pool).await?;

        Ok(Database {
            dead_drop_queries: DeadDropQueries::new(pool.clone()),
            journalist_queries: JournalistQueries::new(pool.clone()),
            covernode_key_queries: CoverNodeKeyQueries::new(pool.clone()),
            hierarchy_queries: HierarchyQueries::new(pool.clone()),
            organization_key_queries: OrganizationKeyQueries::new(pool.clone()),
            system_key_queries: SystemKeyQueries::new(pool.clone()),
            system_queries: SystemQueries::new(pool),
        })
    }
}
