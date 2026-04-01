use super::queries::*;
use anyhow::Context;
use reqwest::Url;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    ConnectOptions,
};

const MAX_DB_CONNECTIONS: u32 = 10;

#[derive(Clone)]
pub struct Database {
    pub client_queries: ClientQueries,
    pub message_queries: MessageQueries,
}

impl Database {
    pub async fn new(db_url: &str) -> anyhow::Result<Database> {
        let url = Url::parse(db_url).expect("Parse db url");

        // We disable statement logging so no connection secrets are sent to logs
        let connect_options = PgConnectOptions::from_url(&url)?.disable_statement_logging();

        let pool = PgPoolOptions::new()
            .max_connections(MAX_DB_CONNECTIONS)
            .connect_with(connect_options)
            .await
            .context("Failed to connect to database")?;

        tracing::info!("Database connection pool created successfully");

        tracing::info!("Running database migrations...");
        sqlx::migrate!()
            .run(&pool)
            .await
            .context("Failed to run database migrations")?;

        tracing::info!("Database migrations completed successfully");

        Ok(Database {
            client_queries: ClientQueries::new(pool.clone()),
            message_queries: MessageQueries::new(pool.clone()),
        })
    }
}
