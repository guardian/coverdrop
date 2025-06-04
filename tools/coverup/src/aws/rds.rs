use crate::aws::ssm::create_tunnel;
use aws_sdk_rds::Client as RdsClient;
use tokio::process::Child;

pub async fn get_rds_hostname(db_name: &String, rds_client: RdsClient) -> anyhow::Result<String> {
    let rds_instances = rds_client
        .describe_db_instances()
        .db_instance_identifier(db_name)
        .send()
        .await?
        .db_instances
        .ok_or_else(|| anyhow::anyhow!("No RDS instances with name {} found", db_name))?;
    let first_instance = rds_instances
        .first()
        .ok_or_else(|| anyhow::anyhow!("No RDS instances with name {}", db_name))?;

    let instance_hostname = first_instance
        .endpoint
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No endpoint found for RDS instance {}", db_name))?
        .address
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No address found for RDS instance {}", db_name))?;

    Ok(instance_hostname.to_string())
}

pub async fn create_db_tunnel(
    db_hostname: String,
    local_port: u16,
    tunnel_instance_id: String,
) -> anyhow::Result<Child> {
    create_tunnel(&db_hostname, "5432", local_port, tunnel_instance_id).await
}
