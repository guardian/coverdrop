use aws_sdk_secretsmanager::types::{Filter as SecretsManagerFilter, FilterNameStringType};
use aws_sdk_secretsmanager::Client as SecretsClient;
use common::clap::Stage;

pub async fn get_secret_arn(
    client: &SecretsClient,
    stage: Stage,
    required_arn_substring: String,
) -> anyhow::Result<String> {
    let secret_response = client
        .list_secrets()
        .filters(
            SecretsManagerFilter::builder()
                .key(FilterNameStringType::TagValue)
                .values("coverdrop")
                .build(),
        )
        .filters(
            SecretsManagerFilter::builder()
                .key(FilterNameStringType::TagValue)
                .values(stage.as_guardian_str())
                .build(),
        )
        .send()
        .await?;

    let secrets = secret_response
        .secret_list
        .ok_or_else(|| anyhow::anyhow!("No secrets found"))?;

    let mut matching_secrets = secrets.iter().filter(|&s| {
        s.arn
            .as_ref()
            .filter(|arn| arn.contains(required_arn_substring.as_str()))
            .is_some()
    });

    let first = matching_secrets.next().ok_or_else(|| {
        anyhow::anyhow!(
            "Failed to find a secret with a tag with value {} and substring {} in the arn",
            stage.as_guardian_str(),
            required_arn_substring
        )
    })?;

    let first_arn = first
        .arn
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Missing arn for secret"))?;

    Ok(first_arn.to_string())
}

pub async fn get_db_password(
    client: SecretsClient,
    secret_arn: String,
    db_name: &String,
) -> anyhow::Result<String> {
    let secret_response = client
        .get_secret_value()
        .secret_id(&secret_arn)
        .send()
        .await?;

    let secret_value = secret_response
        .secret_string
        .clone()
        .ok_or_else(|| anyhow::anyhow!("Secret {} not found", &secret_arn));
    match secret_value {
        Ok(s) => {
            let secret: serde_json::Value = serde_json::from_str(&s)?;
            let db_instance_id = secret["dbInstanceIdentifier"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("DB instance identifier not found in secret"))?;
            if db_instance_id != db_name {
                anyhow::bail!("Secret does not match the expected db name")
            }
            Ok(secret["password"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("No password found in secret"))?
                .to_string())
        }
        Err(e) => {
            anyhow::bail!("Failed to get secret value, error: {:?}", e);
        }
    }
}
