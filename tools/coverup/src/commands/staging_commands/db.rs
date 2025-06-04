use aws_config::SdkConfig;
use common::clap::Stage;

use crate::aws::ec2::get_random_instance;
use crate::aws::rds::{create_db_tunnel, get_rds_hostname};
use crate::aws::secrets::{get_db_password, get_secret_arn};
use reqwest::Url;
use sqlx::postgres::PgPoolOptions;
use sqlx::Executor;

async fn run_wipe_query(db_url: &str) -> anyhow::Result<()> {
    let url = Url::parse(db_url).expect("Parse db url");

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(url.as_str())
        .await?;

    let mut conn = pool.acquire().await?;

    // rather than keep track of all the tables, just drop and recreate the entire schema
    let wipe_tables_query = r#"
        DROP SCHEMA public CASCADE;
        CREATE SCHEMA public;
        GRANT ALL ON SCHEMA public TO coverdrop;
        GRANT ALL ON SCHEMA public TO public;
     "#;

    conn.execute(wipe_tables_query).await?;

    Ok(())
}

pub async fn wipe_all_tables(config: &SdkConfig, stage: Stage) -> anyhow::Result<()> {
    assert_eq!(stage, Stage::Staging);

    println!("Wiping all tables in staging environment");

    let ec2_client = aws_sdk_ec2::Client::new(config);
    let rds_client = aws_sdk_rds::Client::new(config);
    let secret_client = aws_sdk_secretsmanager::Client::new(config);

    let first_api_instance = get_random_instance("api", stage, &ec2_client).await?;
    let first_api_instance_id = first_api_instance
        .instance_id
        .ok_or_else(|| anyhow::anyhow!("No instance id found"))?;
    println!("API instance: {:?}", first_api_instance_id);

    let db_name = format!(
        "coverdrop-api-db-{}",
        stage.as_guardian_str().to_lowercase()
    );
    let api_db_hostname = get_rds_hostname(&db_name, rds_client).await?;
    let local_db_port = 15433;

    let mut tunnel_process =
        create_db_tunnel(api_db_hostname, local_db_port, first_api_instance_id).await?;

    let secret_arn = get_secret_arn(&secret_client, stage, "DatabaseSec".to_string()).await?;
    let db_password = get_db_password(secret_client, secret_arn, &db_name).await?;

    let db_url = format!(
        "postgres://coverdrop:{}@localhost:{}/coverdrop_api_db",
        db_password, local_db_port
    );
    run_wipe_query(&db_url).await?;
    tunnel_process.kill().await.expect("Kill tunnel process");
    tunnel_process.wait().await?;
    println!("Finished table wipe");
    Ok(())
}
