use std::env::set_var;
use std::path::Path;
use std::time::Duration;

use crate::coverdrop_service::CoverDropService;
use bytes::Bytes;
use http::Request;
use http_body_util::{BodyExt, Empty};
use hyper::client::conn::http1::handshake;
use hyper_util::rt::TokioIo;
use k8s_openapi::api::{apps::v1::Deployment, core::v1::Pod};
use kube::{
    api::ListParams,
    runtime::{conditions::is_pod_running, reflector::Lookup, wait::await_condition},
    Api, Client,
};
use tokio::time::timeout;

pub enum CoverDropNamespace {
    OnPremises,
    Cloud,
}

pub struct NamespacedClients {
    pub pods: Api<Pod>,
    pub deployments: Api<Deployment>,
}

pub struct KubeClient {
    pub on_premises: NamespacedClients,
    pub cloud: NamespacedClients,
}

const ON_PREMISES_NAMESPACE: &str = "on-premises";
const CLOUD_NAMESPACE: &str = "cloud";

impl KubeClient {
    pub async fn new(kubeconfig_path: &Option<impl AsRef<Path>>) -> anyhow::Result<Self> {
        if let Some(kubeconfig_path) = kubeconfig_path {
            set_var("KUBECONFIG", kubeconfig_path.as_ref().as_os_str());
        }

        let client = Client::try_default().await?;

        let on_premises = NamespacedClients {
            pods: Api::namespaced(client.clone(), ON_PREMISES_NAMESPACE),
            deployments: Api::namespaced(client.clone(), ON_PREMISES_NAMESPACE),
        };

        let cloud = NamespacedClients {
            pods: Api::namespaced(client.clone(), CLOUD_NAMESPACE),
            deployments: Api::namespaced(client, CLOUD_NAMESPACE),
        };

        let client = Self { on_premises, cloud };

        Ok(client)
    }

    pub fn get_client_for_service(&self, service: &CoverDropService) -> &NamespacedClients {
        match service.to_namespace() {
            CoverDropNamespace::OnPremises => &self.on_premises,
            CoverDropNamespace::Cloud => &self.cloud,
        }
    }

    pub fn get_pod_client_for_serivce(&self, service: &CoverDropService) -> &Api<Pod> {
        &self.get_client_for_service(service).pods
    }

    pub fn get_deployment_client_for_serivce(
        &self,
        service: &CoverDropService,
    ) -> &Api<Deployment> {
        &self.get_client_for_service(service).deployments
    }

    /// Forward a single HTTP request to a pod
    pub async fn forward_http_get_request(
        &self,
        service: CoverDropService,
        uri: &str,
    ) -> anyhow::Result<serde_json::Value> {
        let lp = ListParams::default().labels(&format!("app={}", service.as_str()));

        let pods_client = match service.to_namespace() {
            CoverDropNamespace::OnPremises => &self.on_premises.pods,
            CoverDropNamespace::Cloud => &self.cloud.pods,
        };

        let pods = pods_client.list(&lp).await?;

        if let Some(pod) = pods.items.last() {
            let Some(pod_name) = pod.name() else {
                anyhow::bail!("CoverNode pod was found using labels, but it does not have a name");
            };

            tracing::info!("Waiting for pod {} to be ready", pod_name);

            _ = timeout(
                Duration::from_secs(120),
                await_condition(pods_client.clone(), &pod_name, is_pod_running()),
            );

            let pod_port = service.port();

            let mut port_forward = pods_client.portforward(&pod_name, &[pod_port]).await?;
            let forwarded_stream = port_forward.take_stream(pod_port).unwrap();

            let (mut sender, connection) = handshake(TokioIo::new(forwarded_stream)).await?;

            tokio::spawn(async move {
                if let Err(e) = connection.await {
                    tracing::error!("Error in connection: {}", e);
                }
            });

            let http_req = Request::builder()
                .uri(uri)
                .header("Connection", "close")
                .header("Host", "127.0.0.1")
                .method("GET")
                .body(Empty::<Bytes>::new())
                .unwrap();

            let response = sender.send_request(http_req).await?;

            let body = response.into_body();

            let body_bytes = body.collect().await?.to_bytes();

            let body_str = std::str::from_utf8(&body_bytes)?;

            let response_json = serde_json::from_str(body_str)?;

            Ok(response_json)
        } else {
            anyhow::bail!("No {} pod found", service.as_str())
        }
    }
}
