use crate::aws::credentials::get_sdk_config;
use crate::aws::ssm::get_file_contents;
use aws_sdk_ec2::types::{Filter as Ec2Filter, Instance};
use aws_sdk_ec2::Client as Ec2Client;
use common::clap::{AwsConfig, Stage};

pub async fn get_random_instance(
    app_tag: &str,
    stage: Stage,
    client: &Ec2Client,
) -> anyhow::Result<Instance> {
    let describe_response = client
        .describe_instances()
        .filters(Ec2Filter::builder().name("tag:App").values(app_tag).build())
        .filters(
            Ec2Filter::builder()
                .name("tag:Stage")
                .values(stage.as_guardian_str())
                .build(),
        )
        //ensure instance running
        .filters(
            Ec2Filter::builder()
                .name("instance-state-name")
                .values("running")
                .build(),
        )
        .send()
        .await?;
    let instance_reservations = describe_response
        .reservations
        .ok_or_else(|| anyhow::anyhow!("No instance reservations found"))?;

    let mut instances: Vec<Instance> = instance_reservations
        .iter()
        .flat_map(|r| r.clone().instances.unwrap_or_default())
        .collect();

    let first_instance = instances
        .pop()
        .ok_or_else(|| anyhow::anyhow!("No instances found"))?;

    Ok(first_instance.clone())
}

pub async fn get_file_contents_from_instance(
    aws_config: AwsConfig,
    ssm_output_bucket: String,
    file_path: &str,
) -> anyhow::Result<String> {
    println!("Getting kubeconfig from remote machine");

    let aws_sdk_config = get_sdk_config(aws_config).await;
    let ec2_client = aws_sdk_ec2::Client::new(&aws_sdk_config);

    let random_instance = get_random_instance("coverdrop-k3s", Stage::Staging, &ec2_client)
        .await?
        .instance_id
        .ok_or_else(|| anyhow::anyhow!("Failed to get id of k3s instance"))?;

    let kubeconfig_string = get_file_contents(
        &aws_sdk_config,
        ssm_output_bucket,
        random_instance,
        file_path,
    )
    .await?;

    Ok(kubeconfig_string)
}
