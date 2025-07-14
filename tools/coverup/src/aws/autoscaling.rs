use aws_sdk_autoscaling::types::Filter as AutoscalingFilter;
use aws_sdk_autoscaling::Client as AsgClient;
use common::clap::Stage;

pub async fn get_asg_name(
    app_tag: &str,
    stage: Stage,
    client: &AsgClient,
) -> anyhow::Result<String> {
    let app_filter = AutoscalingFilter::builder()
        .name("tag:App")
        .values(app_tag)
        .build();

    let stage_filter = AutoscalingFilter::builder()
        .name("tag:Stage")
        .values(stage.as_guardian_str())
        .build();

    let response = client
        .describe_auto_scaling_groups()
        .filters(app_filter)
        .filters(stage_filter)
        .send()
        .await?;

    let groups = response
        .auto_scaling_groups
        .ok_or_else(|| anyhow::anyhow!("No auto scaling groups found"))?;

    let first_group = groups
        .first()
        .ok_or_else(|| anyhow::anyhow!("Auto scaling groups list is empty"))?;

    let first_group_name = first_group
        .auto_scaling_group_name
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("Auto scaling group name missing"))
        .map(|s| s.to_string());
    first_group_name
}

pub async fn scale_asg(
    asg_name: &str,
    desired_capacity: i32,
    client: &AsgClient,
) -> anyhow::Result<()> {
    client
        .update_auto_scaling_group()
        .auto_scaling_group_name(asg_name)
        .min_size(desired_capacity)
        .desired_capacity(desired_capacity)
        .send()
        .await?;

    println!("ASG {asg_name:?} scaled to 0 instances");

    Ok(())
}
