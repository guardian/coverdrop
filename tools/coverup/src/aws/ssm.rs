use crate::aws::s3::get_object_as_string;
use crate::subprocess::create_subprocess;
use crate::util::port_in_use;
use aws_config::SdkConfig;
use aws_sdk_ssm::client::Waiters;
use aws_sdk_ssm::types::{CommandInvocation, Target};
use aws_sdk_ssm::Client as SsmClient;
use std::time::Duration;
use tokio::process::Child;

pub async fn create_tunnel(
    remote_host: &str,
    remote_port: &str,
    local_port: u16,
    tunnel_instance_id: String,
) -> anyhow::Result<Child> {
    if port_in_use(local_port).await {
        anyhow::bail!(
            "Port {} is already in use. Please choose a different port or find and kill the existing process (`lsof -i :{}`)",
            local_port, local_port
        );
    }
    println!("Creating tunnel to {remote_host}:{remote_port} at localhost:{local_port}");
    // use aws cli to start session (I tried using the Rust SDK but that doesn't handle the port forwarding)
    let command = format!(
        r##"aws ssm start-session \
            --target {tunnel_instance_id} \
            --document-name "AWS-StartPortForwardingSessionToRemoteHost" \
            --parameters '{{"localPortNumber":["{local_port}"],"portNumber":["{remote_port}"],"host":["{remote_host}"]}}' \
            --profile secure-collaboration \
            --region eu-west-1
            "##
    );
    let child = create_subprocess("Tunnel", command.as_str(), true).await?;

    let child_id = child
        .id()
        .ok_or_else(|| anyhow::anyhow!("Failed to get child process id"))?;
    println!("Tunnel has been created. Tunnel process id: {child_id:?}");

    Ok(child)
}

async fn command_invocations(
    ssm_client: &SsmClient,
    command_id: &str,
) -> anyhow::Result<Vec<CommandInvocation>> {
    let command_invocation_output = ssm_client
        .list_command_invocations()
        .command_id(command_id)
        .details(true)
        .send()
        .await?;
    let command_invocations = command_invocation_output
        .command_invocations
        .ok_or_else(|| anyhow::anyhow!("Failed to get command invocations"))?;
    Ok(command_invocations)
}

pub async fn get_file_contents(
    aws_sdk_config: &SdkConfig,
    ssm_output_bucket: String,
    instance_id: String,
    file_path: &str,
) -> anyhow::Result<String> {
    println!("Fetching config from {instance_id}");
    let ssm_client = aws_sdk_ssm::Client::new(aws_sdk_config);
    let s3_client = aws_sdk_s3::Client::new(aws_sdk_config);
    let send_command = ssm_client
        .send_command()
        .document_name("AWS-RunShellScript")
        .targets(
            Target::builder()
                .set_key(Some("InstanceIds".to_string()))
                .set_values(Some(vec![instance_id.to_string()]))
                .build(),
        )
        .parameters("commands", vec![format!("cat {}", file_path)])
        .output_s3_bucket_name(ssm_output_bucket.to_string())
        .output_s3_key_prefix("ssmoutput")
        .send()
        .await?;
    let command = send_command.command.ok_or_else(|| {
        anyhow::anyhow!("Failed to retrieve command from ssm send command request")
    })?;
    let command_id = command
        .command_id
        .ok_or_else(|| anyhow::anyhow!("Could not get command id of ssm command"))?;
    ssm_client
        .wait_until_command_executed()
        .command_id(&command_id)
        .instance_id(instance_id)
        .wait(Duration::from_secs(300))
        .await?;
    let command_invocations = command_invocations(&ssm_client, &command_id).await?;
    let first_command_invocation = command_invocations
        .first()
        .ok_or_else(|| anyhow::anyhow!("Failed to get first command invocation"))?;

    let stdout_s3_url = first_command_invocation
        .standard_output_url
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Failed to get stdout s3 url"))?;

    let s3_key = stdout_s3_url
        .split(format!("{ssm_output_bucket}/").as_str())
        .last()
        .ok_or_else(|| anyhow::anyhow!("Failed to get s3 key"))?;

    let file_contents = get_object_as_string(s3_client, ssm_output_bucket, s3_key).await?;

    Ok(file_contents)
}
