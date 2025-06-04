use std::path::Path;
use std::time::Duration;

use kube::{api::ListParams, runtime::reflector::Lookup};
use tokio::sync::broadcast::{self, Receiver, Sender};

use crate::{
    coverdrop_service::CoverDropService, dev::watch::status::ServicePodStatus,
    kube_client::KubeClient,
};

use super::k8s_signal::K8sSignal;

pub struct K8sWatcher {
    k8s_signal_tx: Sender<K8sSignal>,
}

impl K8sWatcher {
    pub fn new() -> Self {
        let (k8s_signal_tx, _k8s_signal_rx) = broadcast::channel(100);

        Self { k8s_signal_tx }
    }

    pub fn subscribe(&self) -> Receiver<K8sSignal> {
        self.k8s_signal_tx.subscribe()
    }

    pub async fn watch(&mut self, kubeconfig_path: impl AsRef<Path>) -> anyhow::Result<()> {
        let kube_client = KubeClient::new(&Some(kubeconfig_path)).await?;

        loop {
            tracing::debug!("Getting pod information");

            for service in CoverDropService::all() {
                let lp = ListParams::default().labels(&format!("app={}", service.as_str()));

                let pods_client = kube_client.get_pod_client_for_serivce(service);

                let pods = pods_client.list(&lp).await?;

                let statuses = pods
                    .items
                    .iter()
                    .map(|pod| {
                        let name = pod
                            .name()
                            .map(|name| name.to_string())
                            .unwrap_or_else(|| "<no name>".to_string());

                        let phase = match &pod.status {
                            Some(status) => status
                                .phase
                                .clone()
                                .unwrap_or_else(|| "<unknown phase>".to_string()),
                            None => "<unknown status>".to_string(),
                        };

                        let is_being_deleted = pod.metadata.deletion_timestamp.is_some();

                        ServicePodStatus {
                            name,
                            phase,
                            is_being_deleted,
                        }
                    })
                    .collect::<Vec<_>>();

                let signal = K8sSignal::PodsStatus(*service, statuses);

                _ = self.k8s_signal_tx.send(signal);
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}
