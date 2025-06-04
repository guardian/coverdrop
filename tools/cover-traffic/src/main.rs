use clap::Parser;
use cli::ContinuousTrafficMode;
use common::aws::kinesis::client::KinesisClient;
use common::aws::ssm::client::SsmClient;
use common::protocol::keys::{anchor_org_pk, AnchorOrganizationPublicKey};
use common::tracing::init_tracing;
use common::{
    api::api_client::ApiClient, time::now, u2j_appender::messaging_client::MessagingClient,
};
use continuous::message_per_hour::MessagesPerHour;
use state::CoverTrafficState;

use crate::burst::send_cover_traffic_in_burst;
use crate::cli::{Cli, TrafficCommand};
use crate::continuous::send_cover_traffic_continuously;

mod burst;
mod cli;
mod continuous;
mod state;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing("cover_traffic=debug,info");

    // parse CLI arguments
    let cli = Cli::parse();

    tracing::info!("Starting with config: {:?}", cli);

    let Cli {
        api_url,
        messaging_url,
        command,
        aws_config,
        kinesis_config,
    } = cli;

    // set up clients
    let api_client = ApiClient::new(api_url);
    let messaging_client = MessagingClient::new(messaging_url);
    let kinesis_client = KinesisClient::new(
        &kinesis_config,
        &aws_config,
        vec![kinesis_config.journalist_stream.clone()],
    )
    .await;

    // We trust the public keys when we first see them for the rest of this service's lifetime.
    // This is good enough for cover traffic but, of course, this should not be done in services
    // that deal with any traffic from or to actual users.
    let keys_and_profiles = api_client.get_public_keys().await?;
    let tofu_trusted_public_keys = keys_and_profiles
        .untrusted_org_pk_iter()
        .flat_map(|org_pk| anchor_org_pk(&org_pk.to_tofu_anchor(), now()))
        .collect::<Vec<AnchorOrganizationPublicKey>>();

    let state = CoverTrafficState::new(tofu_trusted_public_keys, keys_and_profiles.keys);

    match command {
        TrafficCommand::Burst { num_u2j, num_j2u } => {
            send_cover_traffic_in_burst(messaging_client, kinesis_client, state, num_u2j, num_j2u)
                .await
        }
        TrafficCommand::Continuous { mode } => {
            let (mph_u2j, mph_j2u) = match mode {
                ContinuousTrafficMode::ParameterStore { parameter_prefix } => {
                    let ssm_client = SsmClient::new(aws_config.region, aws_config.profile).await;

                    let mph_u2j_parameter = parameter_prefix
                        .get_parameter("continuous-messages-per-hour-user-to-journalist");
                    let mph_u2j =
                        MessagesPerHour::new_for_parameter_store(&ssm_client, &mph_u2j_parameter)
                            .await?;

                    let mph_j2u_parameter = parameter_prefix
                        .get_parameter("continuous-messages-per-hour-journalist-to-user");
                    let mph_j2u =
                        MessagesPerHour::new_for_parameter_store(&ssm_client, &mph_j2u_parameter)
                            .await?;

                    (mph_u2j, mph_j2u)
                }
                ContinuousTrafficMode::Manual { mph_u2j, mph_j2u } => {
                    let mph_u2j = MessagesPerHour::new_for_manual(mph_u2j);
                    let mph_j2u = MessagesPerHour::new_for_manual(mph_j2u);
                    (mph_u2j, mph_j2u)
                }
            };

            send_cover_traffic_continuously(
                api_client,
                messaging_client,
                kinesis_client,
                state,
                mph_u2j,
                mph_j2u,
            )
            .await
        }
    }
}
