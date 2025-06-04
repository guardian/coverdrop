use std::env;

use chrono::Days;
use clap::Parser;
use common::{
    api::api_client::ApiClient,
    aws::{
        ses::client::{SendEmailConfig, SesClient},
        ssm::client::SsmClient,
    },
    protocol::keys::{load_anchor_org_pks, load_anchor_org_pks_from_ssm},
    time::{self, now},
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
        key_location,
        team_email_address,
    } = Cli::parse();

    tracing::info!(
        "parameter_prex={:?}, keys_path={:?}, api_url={}",
        key_location.parameter_prefix,
        key_location.keys_path,
        api_url,
    );

    let api_client = ApiClient::new(api_url);

    let (from_email_address, anchor_org_pks) =
        if let Some(prefix) = key_location.parameter_prefix.clone() {
            let ssm_client = SsmClient::new_in_aws().await;
            let from_email_address = source_email(&ssm_client, &prefix).await?;
            let anchor_org_pks = load_anchor_org_pks_from_ssm(&ssm_client, &prefix, now()).await?;
            (from_email_address, anchor_org_pks)
        } else {
            let from_email_address = "test@test.test".to_owned();
            let anchor_org_pks = load_anchor_org_pks(key_location.keys_path.unwrap(), now())?;
            (from_email_address, anchor_org_pks)
        };

    let keys_and_profiles = api_client
        .get_public_keys()
        .await?
        .into_trusted(&anchor_org_pks, time::now());

    let keys = keys_and_profiles.keys;

    let expiring_organization_pk = check_pk(keys.latest_org_pk(), &[Days::new(1), Days::new(14)]);

    let expiring_covernode_provisioning_pk = check_pk(
        keys.latest_covernode_provisioning_pk(),
        &[Days::new(1), Days::new(14)],
    );

    let expiring_journalist_provisioning_pk = check_pk(
        keys.latest_journalist_provisioning_pk(),
        &[Days::new(1), Days::new(14)],
    );

    let covernode_ids = keys.covernode_id_iter().collect::<Vec<_>>();
    let expiring_covernode_id_pks = check_pks_with_identifiers(
        &covernode_ids,
        keys.latest_covernode_id_pk_iter(),
        &[Days::new(1), Days::new(2), Days::new(14)],
    );

    let expiring_covernode_msg_pks = check_pks_with_identifiers(
        &covernode_ids,
        keys.latest_covernode_msg_pk_iter(),
        &[Days::new(1), Days::new(2), Days::new(7)],
    );

    let journalist_ids = keys.journalist_id_iter().collect::<Vec<_>>();
    let expiring_journalist_id_pks = check_pks_with_identifiers(
        &journalist_ids,
        keys.latest_journalist_id_pk_iter(),
        &[Days::new(1), Days::new(2), Days::new(14)],
    );

    let expiring_journalist_msg_pks = check_pks_with_identifiers(
        &journalist_ids,
        keys.latest_journalist_msg_pk_iter(),
        &[Days::new(1), Days::new(2), Days::new(7)],
    );

    if let Some(email_body) = create_email_body(
        expiring_organization_pk,
        expiring_covernode_provisioning_pk,
        expiring_journalist_provisioning_pk,
        &expiring_covernode_id_pks,
        &expiring_covernode_msg_pks,
        &expiring_journalist_id_pks,
        &expiring_journalist_msg_pks,
    ) {
        let in_aws = key_location.parameter_prefix.is_some();
        if in_aws {
            let email_client = SesClient::new_in_aws(from_email_address).await;

            let email = SendEmailConfig {
                to: team_email_address.clone(),
                reply_to: team_email_address.clone(),
                subject: "🚨 Expiring key notification".to_owned(),
                body: email_body,
            };

            email_client.send_email(email).await?;
        } else {
            tracing::info!("{}", email_body);
        }
    }
    Ok(())
}
