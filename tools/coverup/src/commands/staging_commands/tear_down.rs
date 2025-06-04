use aws_config::SdkConfig;

use crate::aws::autoscaling::{get_asg_name, scale_asg};
use crate::aws::credentials::get_sdk_config;
use crate::commands::staging_commands::db::wipe_all_tables;
use common::clap::{AwsConfig, Stage};

async fn scale_down_asg(app_tag: &str, stage: Stage, config: &SdkConfig) -> anyhow::Result<()> {
    println!(
        "Scaling down ASG with app tag: {:?}, stage tag: {:?}",
        app_tag,
        stage.as_guardian_str()
    );

    let autoscaling_client: aws_sdk_autoscaling::Client = aws_sdk_autoscaling::Client::new(config);
    let asg_name = get_asg_name(app_tag, stage, &autoscaling_client).await?;

    println!("Scaling down asg: {:?}", asg_name);

    // scale down api asg to 0 instances
    scale_asg(&asg_name, 0, &autoscaling_client).await?;

    Ok(())
}

pub async fn tear_down(aws_config: AwsConfig) -> anyhow::Result<()> {
    println!("Tearing down the staging environment");
    let stage = Stage::Staging;

    let aws_sdk_config = get_sdk_config(aws_config).await;

    wipe_all_tables(&aws_sdk_config, stage).await?;

    scale_down_asg("api", stage, &aws_sdk_config).await?;
    scale_down_asg("coverdrop-k3s", stage, &aws_sdk_config).await?;

    // This will need updating following https://github.com/guardian/coverdrop/pull/2255
    let ssm = aws_sdk_ssm::Client::new(&aws_sdk_config);
    ssm.delete_parameter()
        .name("/STAGING/secure-collaboration/coverdrop/keys/organization.pub.json")
        .send()
        .await?;

    Ok(())
}
