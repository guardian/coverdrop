use std::env;

use clap::Parser;
use common::{
    api::api_client::ApiClient,
    aws::{
        ses::client::{SendEmailConfig, SesClient},
        ssm::client::SsmClient,
    },
    clap::Stage,
    time::{self},
};

use crate::{
    cli::Cli,
    email::{create_email_body, source_email},
    key_monitors::{check_pk, check_pks_with_identifiers},
};

mod cli;
mod email;
mod expiry_state;
mod key_monitors;

fn init_tracing() {
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "debug")
    }

    tracing_subscriber::fmt()
        // Disabling time is handy because CloudWatch will add the ingestion time
        .without_time()
        // This needs to be set to false, otherwise ANSI color codes will
        // show up in a confusing manner in CloudWatch logs
        .with_ansi(false)
        .init();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let Cli {
        api_url,
        team_email_address,
        stage,
        parameter_prefix,
    } = Cli::parse();

    tracing::info!(
        "parameter_prefix={:?}, api_url={}",
        parameter_prefix,
        api_url,
    );

    let api_client = ApiClient::new(api_url);

    let from_email_address = if let Some(prefix) = parameter_prefix.clone() {
        let ssm_client = SsmClient::new_in_aws().await;
        source_email(&ssm_client, &prefix).await?
    } else if stage == Stage::Development {
        "test@test.test".to_owned()
    } else {
        anyhow::bail!("Couldn't find parameter prefix for source email address")
    };

    let trust_anchors = trust_anchors::get_trust_anchors(&stage, time::now())?;

    let keys_and_profiles = api_client
        .get_public_keys()
        .await?
        .into_trusted(&trust_anchors, time::now());

    let keys = keys_and_profiles.keys;

    let expiring_organization_pk = check_pk(keys.latest_org_pk());

    let expiring_covernode_provisioning_pk = check_pk(keys.latest_covernode_provisioning_pk());

    let expiring_journalist_provisioning_pk = check_pk(keys.latest_journalist_provisioning_pk());

    let covernode_ids = keys.covernode_id_iter().collect::<Vec<_>>();
    let expiring_covernode_id_pks =
        check_pks_with_identifiers(&covernode_ids, keys.latest_covernode_id_pk_iter());

    let expiring_covernode_msg_pks =
        check_pks_with_identifiers(&covernode_ids, keys.latest_covernode_msg_pk_iter());

    let journalist_ids = keys.journalist_id_iter().collect::<Vec<_>>();
    let expiring_journalist_id_pks =
        check_pks_with_identifiers(&journalist_ids, keys.latest_journalist_id_pk_iter());

    let expiring_journalist_msg_pks =
        check_pks_with_identifiers(&journalist_ids, keys.latest_journalist_msg_pk_iter());

    let sentinel_ids = keys.sentinel_id_iter().collect::<Vec<_>>();
    let expiring_sentinel_id_pks =
        check_pks_with_identifiers(&sentinel_ids, keys.latest_sentinel_id_pk_iter());

    if let Some(email_body) = create_email_body(
        expiring_organization_pk,
        expiring_covernode_provisioning_pk,
        expiring_journalist_provisioning_pk,
        expiring_covernode_id_pks,
        expiring_covernode_msg_pks,
        expiring_journalist_id_pks,
        expiring_journalist_msg_pks,
        expiring_sentinel_id_pks,
    ) {
        let in_aws = parameter_prefix.is_some();
        if in_aws {
            let email_client = SesClient::new_in_aws(from_email_address).await;

            let email = SendEmailConfig {
                to: team_email_address.clone(),
                reply_to: team_email_address.clone(),
                subject: format!(
                    "🚨 Key rotation & expiry notification {}",
                    stage.as_guardian_str()
                ),
                body: email_body,
            };

            email_client.send_email(email).await?;
        } else {
            tracing::info!("{}", email_body);
        }
    }
    Ok(())
}
