mod builder_signal;

pub use builder_signal::BuilderSignal;

use std::{collections::HashSet, net::Ipv4Addr, time::Duration};

use kube::{
    api::{Patch, PatchParams},
    runtime::wait::await_condition,
};
use tokio::{
    sync::broadcast::{self, Receiver, Sender},
    time::timeout,
};

use crate::{
    coverdrop_service::CoverDropService,
    dev::{
        build::{
            cargo::cargo_metadata,
            docker::{copy_image_to_node, docker_build_rust},
        },
        watch::{fs::FsSignal, status::BuildStatus},
    },
    k8s::conditions::is_rollout_restart_complete,
    kube_client::KubeClient,
    log_handler::LogHandler,
    multipass::list_coverdrop_nodes,
};

pub struct Builder {
    fs_rx: Receiver<FsSignal>,
    builder_tx: Sender<BuilderSignal>,
    multipass_nodes: Vec<Ipv4Addr>,
    kube_client: KubeClient,
}

impl Builder {
    pub fn new(kube_client: KubeClient, fs_signal_rx: Receiver<FsSignal>) -> anyhow::Result<Self> {
        let multipass_nodes: Vec<Ipv4Addr> = list_coverdrop_nodes()?
            .iter()
            .flat_map(|node| node.local_ip())
            .cloned()
            .collect();

        if multipass_nodes.is_empty() {
            anyhow::bail!("No multipass nodes found, are you running a development cluster?");
        }

        tracing::debug!(
            "Found {} multipass node IPs: {:?}",
            multipass_nodes.len(),
            multipass_nodes
        );

        let (builder_tx, _builder_rx) = broadcast::channel(100);

        Ok(Self {
            kube_client,
            fs_rx: fs_signal_rx,
            builder_tx,
            multipass_nodes,
        })
    }

    pub fn subscribe(&self) -> Receiver<BuilderSignal> {
        self.builder_tx.subscribe()
    }

    pub async fn start(&mut self) {
        // We debounce the work to allow multiple changes to be processed in one go
        let mut debounced_work = DebouncedWork::Waiting(HashSet::new());

        loop {
            debounced_work = debounced_work.process(self).await;
        }
    }
}

enum DebouncedWork {
    Waiting(HashSet<CoverDropService>),
    Working(HashSet<CoverDropService>),
}

const DEBOUNCE_DURATION: Duration = Duration::from_secs(3);

impl DebouncedWork {
    pub async fn process(self, builder: &mut Builder) -> Self {
        let builder_tx = &mut builder.builder_tx;

        let signal_set_status = |service: &CoverDropService, status: BuildStatus| {
            _ = builder_tx.send(BuilderSignal::Status(*service, status));
        };

        let signal_failure = |service: &CoverDropService| {
            _ = builder_tx.send(BuilderSignal::Status(*service, BuildStatus::Idle));
            _ = builder_tx.send(BuilderSignal::Failed(*service));
        };

        match self {
            DebouncedWork::Waiting(mut services) => {
                tokio::select! {
                    Ok(signal) = builder.fs_rx.recv() => {
                        tracing::debug!("Got signal from FS watcher: {:?}", signal);
                        // We've found some work, add it to the list and wait for more to arrive
                        match signal {
                            FsSignal::Dirty(cover_drop_service) => {
                                services.insert(cover_drop_service);
                                DebouncedWork::Waiting(services)
                            }
                        }
                    },
                    _ = tokio::time::sleep(DEBOUNCE_DURATION) => {
                        tracing::debug!("Slept builder for {} seconds, checking for work", DEBOUNCE_DURATION.as_secs());

                        if !services.is_empty() {
                            tracing::debug!("There's work to do!");
                            DebouncedWork::Working(services)
                        } else {
                            tracing::debug!("No work");
                            DebouncedWork::Waiting(services)
                        }
                    }
                }
            }
            DebouncedWork::Working(mut services) => {
                tracing::debug!("Got work: {:?}", services);

                'service_loop: for service in services.iter() {
                    tracing::info!("Building {:?}", service);

                    _ = builder_tx.send(BuilderSignal::Begin(*service));

                    // Pick out the correct client for the service
                    let deployments = builder
                        .kube_client
                        .get_deployment_client_for_serivce(service);

                    // Need to refetch metadata because it might have changed
                    let metadata = match cargo_metadata().await {
                        Ok(m) => m,
                        Err(e) => {
                            tracing::error!("Failed to get cargo metadata: {:?}", e);
                            signal_failure(service);
                            continue;
                        }
                    };

                    signal_set_status(service, BuildStatus::Building);

                    let mut logger_tx = builder_tx.clone();
                    let log_handler = LogHandler::ForwardLogForBuilder(*service, &mut logger_tx);

                    let image_and_tag =
                        match docker_build_rust(&metadata.workspace_root, service, &log_handler)
                            .await
                        {
                            Ok(i) => i,
                            Err(e) => {
                                tracing::error!("Failed to build docker image: {:?}", e);
                                signal_failure(service);
                                continue;
                            }
                        };

                    tracing::info!(
                        "Built {} docker image: {}",
                        service.as_str(),
                        &image_and_tag
                    );

                    //
                    // Todo we can probe the state of the pods here to see if there's a difference
                    // in the image we just built and the one running to skip over copying.
                    //

                    signal_set_status(service, BuildStatus::CopyingImageToClusterNodes);

                    for node_ip in &builder.multipass_nodes {
                        if let Err(e) =
                            copy_image_to_node(&image_and_tag, node_ip, &log_handler).await
                        {
                            tracing::error!(
                                "Failed to copy image {} to node {}: {:?}",
                                image_and_tag,
                                node_ip,
                                e
                            );
                            signal_failure(service);
                            continue 'service_loop;
                        }
                    }

                    tracing::info!(
                        "Pushed {} to nodes: {:?}",
                        image_and_tag,
                        builder.multipass_nodes
                    );

                    signal_set_status(service, BuildStatus::Deploying);

                    let deployment_name = service.as_deployment_str();

                    if let Err(e) = deployments
                        .patch(
                            deployment_name,
                            &PatchParams::default(),
                            &Patch::Strategic(serde_json::json!({
                                "spec": {
                                    "template": {
                                        "spec": {
                                            "containers": [{
                                                "name": service.as_str(),
                                                "image": image_and_tag
                                            }]
                                        }
                                    }
                                }
                            })),
                        )
                        .await
                    {
                        tracing::error!("Failed to update {} image: {:?}", deployment_name, e);
                        signal_failure(service);
                        continue;
                    }

                    tracing::info!("Patched deployment: {}", deployment_name);

                    if let Err(e) = deployments.restart(deployment_name).await {
                        tracing::error!("Failed to restart {}: {:?}", deployment_name, e);
                        signal_failure(service);
                        continue;
                    }

                    tracing::info!("Restarted deployment: {}", deployment_name);

                    signal_set_status(service, BuildStatus::Restarting);

                    tracing::info!("Waiting for rollout to be finished");

                    let timeout = timeout(
                        Duration::from_secs(120),
                        await_condition(
                            deployments.clone(),
                            deployment_name,
                            is_rollout_restart_complete(),
                        ),
                    )
                    .await;

                    signal_set_status(service, BuildStatus::Idle);

                    if let Err(e) = timeout {
                        tracing::error!("Did not complete after 120 seconds: {:?}", e);
                        signal_failure(service);
                    } else {
                        _ = builder_tx.send(BuilderSignal::Success(*service));
                    }
                }

                services.clear();
                DebouncedWork::Waiting(services)
            }
        }
    }
}
