use chrono::{DateTime, Utc};
use client::commands::user::messages::send_user_to_journalist_cover_message;
use common::api::models::covernode_id::CoverNodeIdentity;
use common::aws::s3::client::S3Client;
use common::clap::Stage::Development;
use common::protocol::backup::get_backup_bucket_name;
use common::service;
use common::task::{RunnerMode, TaskApiClient, TASK_RUNNER_API_PORT};
use itertools::Itertools;
use journalist_vault::JournalistVault;
use log::info;
use std::fs;
use std::io::Write;
use std::net::IpAddr;
use std::{
    fs::File,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use testcontainers::core::Host::Addr;
use testcontainers::core::{CmdWaitFor, ExecCommand};
use testcontainers::ContainerAsync;
use uuid::Uuid;

use common::clap::{AwsConfig, KinesisConfig};
use common::time::now;
use common::{
    api::api_client::ApiClient, aws::kinesis::client::KinesisClient,
    identity_api::client::IdentityApiClient, u2j_appender::messaging_client::MessagingClient,
};
use reqwest::Url;

use crate::api_wrappers::trigger_load_org_pk_api;
use crate::constants::{MINIO_PORT, U2J_APPENDER_PORT, VARNISH_PORT};
use crate::containers::minio::start_minio;
use crate::containers::u2j_appender::start_u2j_appender;
use crate::containers::varnish::start_varnish;
use crate::images::{Minio, U2JAppender, Varnish};
use crate::keys::{ensure_key_permissions, open_covernode_database, CoverNodeKeyMode};
use crate::secrets::{
    do_secrets_exist_in_container_logs, API_AWS_ACCESS_KEY_ID_SECRET,
    API_AWS_SECRET_ACCESS_KEY_SECRET, MAILBOX_PASSWORD,
};
use crate::{
    constants::{API_PORT, IDENTITY_API_PORT, KINESIS_PORT},
    containers::{
        api::start_api, covernode::start_covernode, identity_api::start_identity_api,
        kinesis::start_kinesis, postgres::start_postgres,
    },
    docker_utils::time_travel_container,
    images::{Api, CoverNode, IdentityApi, Kinesis, Postgres},
    keys::{
        add_stack_keys_to_api, get_keys_generated_at_time, get_static_keys_path,
        load_static_stack_keys, StackKeys,
    },
    mailboxes::{load_mailboxes, StackMailboxes},
};
use covernode_database::Database;
use tempfile::{tempdir_in, TempDir};
use tokio::time::sleep;

/// A full stack represents a full deployment of the system, including the API, Kinesis, Postgres and the Covernode.
pub struct CoverDropStack {
    // Services
    _kinesis: ContainerAsync<Kinesis>,
    postgres: ContainerAsync<Postgres>,
    _u2j_appender: ContainerAsync<U2JAppender>,
    covernode: ContainerAsync<CoverNode>,
    api: ContainerAsync<Api>,
    identity_api: ContainerAsync<IdentityApi>,
    _varnish_cache: ContainerAsync<Varnish>,
    _minio: ContainerAsync<Minio>,

    // Local state,
    temp_dir: TempDir,
    keys_path: PathBuf,
    api_keys_path: PathBuf,
    stack_keys: StackKeys,
    mailboxes: StackMailboxes,
    _checkpoints_dir: TempDir,
    covernode_database: Database,

    // Time management
    base_time: DateTime<Utc>,
    current_time: DateTime<Utc>,
    stopwatch: Instant,

    // Clients
    messaging_client: MessagingClient,

    api_client_cached: ApiClient,
    api_client_uncached: ApiClient,
    _api_task_api_client: TaskApiClient<service::Api>,

    identity_api_client: IdentityApiClient,
    identity_api_task_api_client: Option<TaskApiClient<service::IdentityApi>>,

    covernode_task_api_client: Option<TaskApiClient<service::CoverNode>>,

    kinesis_client: KinesisClient,

    s3_client: S3Client,

    covernode_id: CoverNodeIdentity,
}

pub struct CoverDropStackBuilder {
    network: String,
    default_journalist_id: Option<String>,
    delete_old_dead_drops_poll_seconds: Option<i64>,
    additional_journalists: Option<u8>,
    varnish_api_cache: bool,
    covernode_key_mode: CoverNodeKeyMode,
    covernode_task_runner_mode: Option<RunnerMode>,
    identity_api_task_runner_mode: Option<RunnerMode>,
    cover_message_sender: bool,
}

impl CoverDropStackBuilder {
    pub fn with_default_journalist_id(mut self, default_journalist_id: &str) -> Self {
        self.default_journalist_id = Some(default_journalist_id.into());
        self
    }

    pub fn with_delete_old_dead_drops_poll_duration(mut self, duration: Duration) -> Self {
        self.delete_old_dead_drops_poll_seconds = Some(duration.as_secs() as i64);
        self
    }

    pub fn with_additional_journalists(mut self, number: u8) -> Self {
        self.additional_journalists = Some(number);
        self
    }

    pub fn with_varnish_api_cache(mut self, use_cache: bool) -> Self {
        // TODO eventually all integration tests should be using the api cache.
        // This function and attribute can be removed once that change has been made.
        self.varnish_api_cache = use_cache;
        self
    }

    pub fn with_covernode_key_mode(mut self, with_covernode_key_mode: CoverNodeKeyMode) -> Self {
        self.covernode_key_mode = with_covernode_key_mode;
        self
    }

    pub fn with_covernode_task_runner_mode(mut self, runner_mode: RunnerMode) -> Self {
        self.covernode_task_runner_mode = Some(runner_mode);
        self
    }

    pub fn with_identity_api_task_runner_mode(mut self, runner_mode: RunnerMode) -> Self {
        self.identity_api_task_runner_mode = Some(runner_mode);
        self
    }

    pub fn with_cover_message_sender(mut self) -> Self {
        self.cover_message_sender = true;
        self
    }

    pub async fn build(self) -> CoverDropStack {
        ensure_key_permissions();

        let source_keys_path = get_static_keys_path();
        let base_time = get_keys_generated_at_time(&source_keys_path);

        let temp_dir = tempdir_in(std::env::current_dir().unwrap()).unwrap();

        let api_key_dir = temp_dir.path().join("api");
        let identity_api_key_dir = temp_dir.path().join("identity-api");
        let covernode_key_dir = temp_dir.path().join("covernode");
        let test_runner_key_dir = temp_dir.path().join("test-runner");

        let all_key_dirs = [
            &api_key_dir,
            &identity_api_key_dir,
            &covernode_key_dir,
            &test_runner_key_dir,
        ];

        for key_dir in all_key_dirs {
            std::fs::create_dir(key_dir).expect("Create key dir");
        }

        //Copy all static keys into the temp dir so that we can do more tests where new keys are added "manually"
        fs::read_dir(&source_keys_path)
            .expect("Read keys_path directory")
            .for_each(|entry| {
                if let Ok(entry) = entry {
                    let file_type = entry.file_type().expect("Get filetype");
                    if file_type.is_file() {
                        for dir in all_key_dirs {
                            fs::copy(entry.path(), dir.join(entry.file_name()))
                                .expect("Copy static keys to temp dir");
                        }
                    }
                }
            });

        let stopwatch = Instant::now();

        // We need to set some AWS credentials for this shell so that we
        // can modify the number of kinesis shards
        // And these need to be set to minio credentials in order to create a bucket below
        std::env::set_var("AWS_ACCESS_KEY_ID", API_AWS_ACCESS_KEY_ID_SECRET);
        std::env::set_var("AWS_SECRET_ACCESS_KEY", API_AWS_SECRET_ACCESS_KEY_SECRET);

        let aws_config = AwsConfig {
            region: "eu-west-1".to_string(),
            profile: None,
        };

        // minio
        let minio = start_minio(&self.network).await;

        let minio_port = minio
            .get_host_port_ipv4(MINIO_PORT)
            .await
            .expect("Get minio port");

        let minio_ip_address = minio
            .get_bridge_ip_address()
            .await
            .expect("Get minio bridge ip address");

        let minio_hostname = "localhost";

        let minio_client_url = format!("http://{}:{}", minio_hostname, minio_port);

        // This is fine when communicating from the tests to s3, but for inter-container communication we need the ip address
        let s3_client = S3Client::new(
            aws_config.clone(),
            Url::parse(&minio_client_url).expect("Parse minio URL"),
        )
        .await;

        // create a default bucket
        let backup_bucket_name = get_backup_bucket_name(&Development);
        s3_client
            .create_bucket(&backup_bucket_name)
            .await
            .expect("Create default minio bucket");

        //
        // Fastly and messaging
        //

        let kinesis = start_kinesis(&self.network).await;

        let kinesis_config = KinesisConfig {
            endpoint: format!(
                "http://localhost:{}",
                kinesis
                    .get_host_port_ipv4(KINESIS_PORT)
                    .await
                    .expect("Get host port for kinesis")
            ),
            user_stream: "user-messages".into(),
            journalist_stream: "journalist-messages".into(),
        };

        let kinesis_client = KinesisClient::new(
            &kinesis_config,
            &aws_config,
            vec![
                kinesis_config.user_stream.clone(),
                kinesis_config.journalist_stream.clone(),
            ],
        )
        .await;
        let kinesis_ip = kinesis
            .get_bridge_ip_address()
            .await
            .expect("Get kinesis bridge ip address");

        let u2j_appender: ContainerAsync<U2JAppender> = start_u2j_appender(
            &self.network,
            kinesis
                .get_bridge_ip_address()
                .await
                .expect("Get bridge ip address for u2j appender"),
        )
        .await;
        let u2j_appender_port = u2j_appender
            .get_host_port_ipv4(U2J_APPENDER_PORT)
            .await
            .expect("Get U2J Appender port port");

        let messaging_url = Url::parse(&format!("http://localhost:{u2j_appender_port}"))
            .expect("Parse U2J Appender URL");

        let messaging_client = MessagingClient::new(messaging_url);

        //
        // Database
        //

        let postgres = start_postgres(&self.network).await;

        //
        // API
        //
        let api = start_api(
            &self.network,
            &api_key_dir,
            postgres
                .get_bridge_ip_address()
                .await
                .expect("Get bridge ip address for postgres"),
            base_time,
            self.delete_old_dead_drops_poll_seconds,
            self.default_journalist_id,
            kinesis_ip,
            minio_client_url,
            Addr(minio_ip_address),
        )
        .await;

        let api_port = api
            .get_host_port_ipv4(API_PORT)
            .await
            .expect("Get api port");

        let api_task_api_port = api
            .get_host_port_ipv4(TASK_RUNNER_API_PORT)
            .await
            .expect("Get api task runner port");

        let api_host = api
            .get_bridge_ip_address()
            .await
            .expect("Get bridge ip address for api");

        let api_task_api_client = TaskApiClient::new(
            Url::parse(&format!("http://localhost:{api_task_api_port}"))
                .expect("Failed to parse covernode task API url"),
        )
        .expect("Create covernode task API client");

        //
        // API varnish cache
        // create two versions of the vcl
        //  - boot: respects cache headers
        //  - nocache: forwards all requests to the API without caching
        // nocache is active by default. opt in to caching using with_varnish_api_cache
        let vcl_path = temp_dir.path().to_path_buf();

        format_and_save_vcl(
            "varnish/api_cache_template.vcl",
            temp_dir.path().join("default.vcl"),
            api_host,
            API_PORT,
        );

        format_and_save_vcl(
            "varnish/api_no_cache_template.vcl",
            temp_dir.path().join("no-cache.vcl"),
            api_host,
            API_PORT,
        );

        let varnish_cache = start_varnish(&self.network, vcl_path).await;
        let varnish_port = varnish_cache
            .get_host_port_ipv4(VARNISH_PORT)
            .await
            .expect("Get varnish port");

        // turn off caching for varnish during setup
        exec_vcl_command(&varnish_cache, "varnish/use_nocache.sh").await;

        let api_url = Url::parse(&format!("http://localhost:{varnish_port}")).unwrap();
        let api_client_cached = ApiClient::new(api_url);

        let api_url_uncached = Url::parse(&format!("http://localhost:{api_port}")).unwrap();
        let api_client_uncached = ApiClient::new(api_url_uncached);

        //
        // identity api
        //
        let identity_api = start_identity_api(
            &self.network,
            &identity_api_key_dir,
            api.get_bridge_ip_address()
                .await
                .expect("Get bridge ip address for identity api"),
            self.identity_api_task_runner_mode
                .unwrap_or(RunnerMode::Timer),
            base_time,
        )
        .await;

        let identity_api_port = identity_api
            .get_host_port_ipv4(IDENTITY_API_PORT)
            .await
            .expect("Get host port for identity api");

        let identity_api_url =
            Url::parse(&format!("http://localhost:{identity_api_port}")).unwrap();

        let identity_api_client = IdentityApiClient::new(identity_api_url);

        let identity_api_task_api_client = if self
            .identity_api_task_runner_mode
            .is_some_and(|m| m.triggerable())
        {
            let identity_api_task_api_port = identity_api
                .get_host_port_ipv4(TASK_RUNNER_API_PORT)
                .await
                .expect("Get identity-api task runner port");

            Some(
                TaskApiClient::new(
                    Url::parse(&format!("http://localhost:{identity_api_task_api_port}"))
                        .expect("Parse identity-api task API url"),
                )
                .expect("Create identity-api task API client"),
            )
        } else {
            None
        };

        //
        // Keys
        //

        let covernode_id = CoverNodeIdentity::from_node_id(1);
        let stack_keys = load_static_stack_keys(base_time);

        // Make sure we've set up our org pk in the API before we start futzing with keys
        trigger_load_org_pk_api(&api_task_api_client).await;

        // Create a covernode database and decide how to insert it's initial keys
        // using the `add_stack_keys_to_api` function. We then drop the database
        // to make sure it is fully flushed to disk before mounting it to a docker container.
        //
        // This seems to help with an issue where the setup bundle cannot be found in the
        // mounted file system but it exists in the host file system.
        {
            let covernode_database = open_covernode_database(&covernode_key_dir, &covernode_id)
                .await
                .expect("Create covernode database");

            add_stack_keys_to_api(
                &stack_keys,
                &api_client_uncached,
                base_time,
                covernode_database.clone(),
                self.covernode_key_mode,
            )
            .await;
        }

        let covernode_database = open_covernode_database(&covernode_key_dir, &covernode_id)
            .await
            .expect("Create covernode database");

        let additional_journalists = self.additional_journalists.unwrap_or(0);
        let mailboxes = load_mailboxes(
            &api_client_cached,
            &get_static_keys_path(),
            &temp_dir,
            additional_journalists,
            &stack_keys.user_key_pair,
            stack_keys.keys_generated_at,
        )
        .await;

        let api_ip = if self.varnish_api_cache {
            varnish_cache
                .get_bridge_ip_address()
                .await
                .expect("Get api bridge ip address")
        } else {
            api.get_bridge_ip_address()
                .await
                .expect("Get api bridge ip address")
        };

        let api_port = if self.varnish_api_cache {
            VARNISH_PORT
        } else {
            API_PORT
        };

        let identity_api_ip = identity_api
            .get_bridge_ip_address()
            .await
            .expect("Get identity api bridge ip address");

        let checkpoints_dir =
            tempdir_in(std::env::current_dir().unwrap()).expect("Create temporary keys directory");

        let covernode = start_covernode(
            covernode_id.clone(),
            &self.network,
            &covernode_key_dir,
            &checkpoints_dir,
            api_ip,
            api_port,
            identity_api_ip,
            kinesis_ip,
            base_time,
            self.covernode_task_runner_mode.unwrap_or(RunnerMode::Timer),
        )
        .await;

        // We only create a task API for the CoverNode when it actually has a triggerable task runner; this hopefully prevents
        // some otherwise hard-to-debug mistakes
        let covernode_task_api_port = covernode
            .get_host_port_ipv4(TASK_RUNNER_API_PORT)
            .await
            .expect("Get covernode task runner port");

        let covernode_task_api_client = if self
            .covernode_task_runner_mode
            .is_some_and(|m| m.triggerable())
        {
            Some(
                TaskApiClient::new(
                    Url::parse(&format!("http://localhost:{covernode_task_api_port}"))
                        .expect("Parse covernode task API url"),
                )
                .expect("Create covernode task API client"),
            )
        } else {
            None
        };

        // turn on caching for varnish for tests
        if self.varnish_api_cache {
            exec_vcl_command(&varnish_cache, "varnish/use_cache.sh").await;
        }

        if self.cover_message_sender {
            tokio::task::spawn({
                let messaging_client = messaging_client.clone();
                let keys_and_profiles = api_client_uncached
                    .get_public_keys()
                    .await
                    .expect("Get keys from API");

                let keys_and_profiles =
                    keys_and_profiles.into_trusted(&stack_keys.anchor_org_pks(), base_time);

                async move {
                    send_user_to_journalist_cover_message(
                        &messaging_client,
                        &keys_and_profiles.keys,
                    )
                    .await
                    .expect("Send user cover message");

                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            });
        }

        let mut stack = CoverDropStack {
            _kinesis: kinesis,
            _minio: minio,
            postgres,
            _u2j_appender: u2j_appender,
            covernode,
            api,
            identity_api,
            _varnish_cache: varnish_cache,
            temp_dir,
            keys_path: test_runner_key_dir,
            api_keys_path: api_key_dir,
            stack_keys,
            mailboxes,
            _checkpoints_dir: checkpoints_dir,
            covernode_database,
            base_time,
            current_time: base_time,
            stopwatch,
            messaging_client,
            api_client_cached,
            api_client_uncached,
            identity_api_client,
            identity_api_task_api_client,
            covernode_task_api_client,
            _api_task_api_client: api_task_api_client,
            kinesis_client,
            s3_client,
            covernode_id,
        };

        // Move the infrastructure's time to be appropriate for the keys
        stack.time_travel(stack.now()).await;

        stack
    }
}

impl CoverDropStack {
    pub async fn new() -> CoverDropStack {
        Self::builder().build().await
    }

    pub fn builder() -> CoverDropStackBuilder {
        CoverDropStackBuilder {
            // We use a random network name so that each stack has a separate network,
            // and stacks started for multiple tests do not share a network
            network: Uuid::new_v4().to_string(),
            delete_old_dead_drops_poll_seconds: None,
            additional_journalists: None,
            default_journalist_id: None,
            varnish_api_cache: false,
            covernode_key_mode: CoverNodeKeyMode::ProvidedKeyPair,
            identity_api_task_runner_mode: None,
            covernode_task_runner_mode: None,
            cover_message_sender: false,
        }
    }

    pub fn messaging_client(&self) -> &MessagingClient {
        &self.messaging_client
    }

    pub fn api_postgres(&self) -> &ContainerAsync<Postgres> {
        &self.postgres
    }

    pub fn covernode(&self) -> &ContainerAsync<CoverNode> {
        &self.covernode
    }

    pub fn kinesis_client(&self) -> &KinesisClient {
        &self.kinesis_client
    }

    pub fn temp_dir_path(&self) -> &Path {
        self.temp_dir.path()
    }

    pub fn keys_path(&self) -> &Path {
        &self.keys_path
    }

    pub fn api_keys_path(&self) -> &Path {
        &self.api_keys_path
    }

    pub fn api_client_cached(&self) -> &ApiClient {
        &self.api_client_cached
    }

    pub fn api_client_uncached(&self) -> &ApiClient {
        &self.api_client_uncached
    }

    pub fn identity_api_client(&self) -> &IdentityApiClient {
        &self.identity_api_client
    }

    pub fn identity_api_task_api_client(&self) -> &TaskApiClient<service::IdentityApi> {
        self.identity_api_task_api_client.as_ref().expect("The Identity API task API client is only available when the Identity APi task runner is triggerable")
    }

    pub fn s3_client(&self) -> &S3Client {
        &self.s3_client
    }

    pub fn covernode_task_api_client(&self) -> &TaskApiClient<service::CoverNode> {
        self.covernode_task_api_client.as_ref().expect("The Covernode task API client is only available when the Covernode task runner is triggerable")
    }

    pub async fn time_travel(&mut self, to: DateTime<Utc>) {
        self.current_time = to;
        self.stopwatch = Instant::now();

        time_travel_container(&self.api, to).await;
        time_travel_container(&self.covernode, to).await;
        time_travel_container(&self.identity_api, to).await;

        // Time travel is async - let's wait a bit to give the docker instances time to catch up.
        sleep(Duration::from_secs(2)).await;

        info!("Time travelled to: {}", now());
    }

    pub fn base_time(&self) -> DateTime<Utc> {
        self.base_time
    }

    pub fn now(&self) -> DateTime<Utc> {
        self.current_time
            + chrono::Duration::from_std(self.stopwatch.elapsed())
                .expect("Convert stopwatch duration to chrono duration")
    }

    pub fn keys(&self) -> &StackKeys {
        &self.stack_keys
    }

    pub fn mailboxes(&self) -> &StackMailboxes {
        &self.mailboxes
    }

    pub fn covernode_database(&self) -> &Database {
        &self.covernode_database
    }

    pub fn covernode_id(&self) -> &CoverNodeIdentity {
        &self.covernode_id
    }

    pub async fn scale_kinesis(&self) {
        self.kinesis_client
            .split_journalist_to_user_shard()
            .await
            .expect("Update journalist to user shard count");

        self.kinesis_client
            .split_user_to_journalist_shard()
            .await
            .expect("Update user to journalist shard count");
    }

    pub async fn do_secrets_exist_in_stack(&self) -> bool {
        //
        // TODO check other services?
        //

        do_secrets_exist_in_container_logs(
            self.covernode(),
            self.temp_dir_path().join("covernode_logs.txt"),
        )
        .await
        .expect("Check covernode logs for secrets")
    }

    pub async fn load_static_journalist_vault(&self) -> JournalistVault {
        JournalistVault::open(
            self.temp_dir_path().join("static_test_journalist.vault"),
            MAILBOX_PASSWORD,
        )
        .await
        .expect("Load static journalist vault")
    }

    pub async fn load_static_journalist_vault_bytes(&self) -> Vec<u8> {
        fs::read(self.temp_dir_path().join("static_test_journalist.vault"))
            .expect("Load static journalist vault")
    }

    pub async fn save_static_journalist_vault_bytes(&self, contents: Vec<u8>) {
        fs::write(
            self.temp_dir_path().join("static_test_journalist.vault"),
            contents,
        )
        .expect("Save static journalist vault")
    }

    pub async fn load_additional_journalist_vault(&self, index: usize) -> JournalistVault {
        JournalistVault::open(
            self.temp_dir_path()
                .join(format!("additional_test_journalist_{index}.vault")),
            MAILBOX_PASSWORD,
        )
        .await
        .expect("Load additional journalist vault")
    }
}

fn format_and_save_vcl(
    vcl_template_relative_path: &str,
    vcl_file_path: PathBuf,
    api_host: IpAddr,
    api_port: u16,
) {
    let mut vcl_file = File::create(vcl_file_path).expect("create vcl config file");

    let vcl_template_full_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(vcl_template_relative_path);

    let vcl_template = fs::read_to_string(vcl_template_full_path).unwrap();
    let vcl = vcl_template
        .replace("{host}", api_host.to_string().as_str())
        .replace("{port}", api_port.to_string().as_str());

    vcl_file.write_all(vcl.as_bytes()).expect("Write vcl file");
}

async fn exec_vcl_command(varnish_cache: &ContainerAsync<Varnish>, command_path: &str) {
    let varnish_no_cache_command_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(command_path);
    let varnish_command = fs::read_to_string(varnish_no_cache_command_path).unwrap();
    let varnish_command_vec = varnish_command.split('\n').collect_vec();

    varnish_cache
        .exec(
            ExecCommand::new(varnish_command_vec).with_cmd_ready_condition(CmdWaitFor::seconds(1)),
        )
        .await
        .expect("Execute cache command");
}
