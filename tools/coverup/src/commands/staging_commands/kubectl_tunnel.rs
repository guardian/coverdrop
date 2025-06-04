use crate::aws::credentials::get_sdk_config;
use crate::aws::ec2::get_random_instance;
use crate::aws::ssm::create_tunnel;
use common::clap::{AwsConfig, Stage};
use tokio::process::Child;

pub async fn kubectl_tunnel(aws_config: AwsConfig, port: u16) -> anyhow::Result<Child> {
    println!("Setting up tunnel to staging k3s cluster");
    let stage = Stage::Staging;

    let aws_sdk_config = get_sdk_config(aws_config).await;
    let ec2_client = aws_sdk_ec2::Client::new(&aws_sdk_config);

    let bastion_instance = get_random_instance("coverdrop-k3s-bastion", stage, &ec2_client).await?;
    let bastion_instance_id = bastion_instance
        .instance_id
        .ok_or_else(|| anyhow::anyhow!("Bastion instance id not found"))?;

    let k3s_instance = get_random_instance("coverdrop-k3s", stage, &ec2_client).await?;
    let k3s_instance_ip = k3s_instance
        .private_ip_address
        .ok_or_else(|| anyhow::anyhow!("K3s instance private ip not found"))?;

    let child = create_tunnel(&k3s_instance_ip, "6443", port, bastion_instance_id).await?;

    Ok(child)
}
