use clap::Parser;
use cli::Cli;
use common::api::api_client::ApiClient;
use common::aws::kinesis::client::KinesisClient;
use common::identity_api::client::IdentityApiClient;
use common::metrics::{init_metrics, COVERNODE_NAMESPACE};
use common::task::{HeartbeatTask, TaskRunner};
use common::time;
use common::tracing::{init_tracing_with_reload_handle, log_task_exit, log_task_result_exit};
use covernode::mixing::mixing_strategy::MixingStrategyConfiguration;
use covernode::services::journalist_to_user_covernode_service::JournalistToUserCoverNodeService;
use covernode::services::tasks::{DeleteExpiredKeysTask, RefreshTagLookUpTableTask};
use covernode::services::user_to_journalist_covernode_service::UserToJournalistCoverNodeService;
use covernode::services::CoverNodeServiceConfig;
use covernode::*;
use covernode_database::Database;
use key_state::KeyState;
use services::server;
use services::tasks::{CreateKeysTask, PublishedKeysTask, TrustedOrganizationPublicKeyPollTask};

mod cli;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let _ = start(&cli).await.map_err(|e| {
        tracing::error!("{}", e);
        if cli.park_on_error {
            tracing::info!("Error in main function, parking application...");
            std::thread::park();
        }
    });
}

async fn start(cli: &Cli) -> anyhow::Result<()> {
    //
    // Initialize tracing
    //
    let _ = init_tracing_with_reload_handle("info");
    init_metrics(COVERNODE_NAMESPACE, &cli.stage).await?;

    tracing::info!("Cli args: {cli:?}");

    let api_client = ApiClient::new(cli.api_url.clone());
    let identity_api_client = IdentityApiClient::new(cli.identity_api_url.clone());

    tracing::info!("Using checkpoint location {:?}", cli.checkpoint_path);
    let checkpoints = load_checkpoints(&cli.checkpoint_path)
        .expect("Read checkpoint files from the specified directory");

    let kinesis_client = KinesisClient::new_with_checkpoints(
        &cli.kinesis_config,
        &cli.aws_config,
        vec![
            cli.kinesis_config.user_stream.clone(),
            cli.kinesis_config.journalist_stream.clone(),
        ],
        checkpoints,
    )
    .await;

    let db = Database::open(&cli.db_path, &cli.db_password).await?;

    tracing::info!("Using key location {:?}", cli.keys_path);
    let key_state = KeyState::new(
        db.clone(),
        &cli.keys_path,
        &cli.parameter_prefix,
        &api_client,
        time::now(),
    )
    .await?;

    tracing::debug!("Setting up background tasks");
    let mut background_tasks = tokio::spawn({
        let create_keys_task = CreateKeysTask::new(
            chrono::Duration::seconds(cli.create_keys_task_period_seconds.get() as i64),
            key_state.clone(),
        );

        let publish_keys_task = PublishedKeysTask::new(
            chrono::Duration::seconds(cli.publish_keys_task_period_seconds.get() as i64),
            key_state.clone(),
            api_client.clone(),
            identity_api_client.clone(),
        );

        let refresh_tag_lookup_table_task = RefreshTagLookUpTableTask::new(
            chrono::Duration::seconds(cli.journalist_cache_refresh_period_seconds.get() as i64),
            key_state.clone(),
        );

        let anchor_org_pk_poll_task = TrustedOrganizationPublicKeyPollTask::new(
            chrono::Duration::seconds(
                cli.anchor_organization_public_key_polling_period_seconds
                    .get() as i64,
            ),
            cli.keys_path.clone(),
            cli.parameter_prefix.clone(),
            key_state.clone(),
        );

        let delete_expired_keys_task = DeleteExpiredKeysTask::new(
            chrono::Duration::seconds(cli.delete_expired_keys_task_period_seconds.get() as i64),
            key_state.clone(),
        );

        let heartbeat_task = HeartbeatTask::default();

        let mut runner = TaskRunner::new(cli.task_runner_mode);

        runner.add_task(refresh_tag_lookup_table_task).await;
        runner.add_task(publish_keys_task).await;
        runner.add_task(create_keys_task).await;
        runner.add_task(anchor_org_pk_poll_task).await;
        runner.add_task(heartbeat_task).await;
        runner.add_task(delete_expired_keys_task).await;

        async move { runner.run().await }
    });

    // run health check before we get into the troubles of setting everything up
    tracing::info!("Using server base URL {:}", cli.api_url);
    let health_check_result = api_client.get_health_check().await;

    match health_check_result {
        Ok(_) => {
            tracing::info!("Health check OK");
        }
        Err(err) => {
            tracing::warn!("Health check failed: {:?}", err);
            return Err(err);
        }
    }

    // cover node service user -> journalist
    let mixing_u2j_config = MixingStrategyConfiguration::new(
        cli.u2j_threshold_min,
        cli.u2j_threshold_max,
        "U2JMixerLevel",
        chrono::Duration::seconds(cli.u2j_timeout_seconds as i64),
        cli.u2j_output_size,
    );
    tracing::debug!("Mixing user->journalist config: {mixing_u2j_config:?}");

    let config_user_to_journalist = CoverNodeServiceConfig {
        api_url: cli.api_url.clone(),
        key_state: key_state.clone(),
        api_client: api_client.clone(),
        checkpoint_path: cli.checkpoint_path.clone(),
        kinesis_client: kinesis_client.clone(),
        mixing_config: mixing_u2j_config,
        disable_stream_throttle: cli.disable_stream_throttle,
    };
    let mut user_to_journalist_service = tokio::spawn(async move {
        let service_user_to_journalist =
            UserToJournalistCoverNodeService::new(config_user_to_journalist);
        service_user_to_journalist.run().await
    });

    // ⚠️ WARNING: DO NOT CHANGE THE LINE BELOW! ⚠️
    // `testcontainers` relies on this line being printed to stdout
    // to determine that the container is ready for integration testing
    tracing::info!("Started CoverNode service user->journalist");

    // start cover node service journalist -> user
    let mixing_j2u_config = MixingStrategyConfiguration::new(
        cli.j2u_threshold_min,
        cli.j2u_threshold_max,
        "J2UMixerLevel",
        chrono::Duration::seconds(cli.j2u_timeout_seconds as i64),
        cli.j2u_output_size,
    );
    tracing::debug!("Mixing journalist->user config: {mixing_j2u_config:?}");

    let config_journalist_to_user = CoverNodeServiceConfig {
        api_url: cli.api_url.clone(),
        key_state: key_state.clone(),
        api_client: api_client.clone(),
        checkpoint_path: cli.checkpoint_path.clone(),
        kinesis_client: kinesis_client.clone(),
        mixing_config: mixing_j2u_config,
        disable_stream_throttle: cli.disable_stream_throttle,
    };
    let mut journalist_to_user_service = tokio::spawn(async move {
        let service_journalist_to_user =
            JournalistToUserCoverNodeService::new(config_journalist_to_user);
        service_journalist_to_user.run().await
    });

    let port = cli.port;
    let mut web_service = tokio::spawn(async move { server::serve(port, key_state).await });

    // ⚠️ WARNING: DO NOT CHANGE THE LINE BELOW! ⚠️
    // `testcontainers` relies on this line being printed to stdout
    // to determine that the container is ready for integration testing
    tracing::info!("Started CoverNode service journalist->user");

    // block until the first service fails/exits; in that case we abort the other
    tokio::select! {
        r = (&mut background_tasks) => {
            log_task_exit("background tasks", r);

            journalist_to_user_service.abort();
            user_to_journalist_service.abort();
            web_service.abort();
        },
        r = (&mut user_to_journalist_service) => {
            log_task_result_exit("U2J service", r);

            background_tasks.abort();
            journalist_to_user_service.abort();
            web_service.abort();
        },
        r = (&mut journalist_to_user_service) => {
            log_task_result_exit("J2U service", r);

            background_tasks.abort();
            user_to_journalist_service.abort();
            web_service.abort();
        },
        r = (&mut web_service) => {
            log_task_result_exit("web service", r);

            background_tasks.abort();
            journalist_to_user_service.abort();
            user_to_journalist_service.abort();
        }
    }

    Ok(())
}
