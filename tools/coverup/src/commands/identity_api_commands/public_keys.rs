use crate::{coverdrop_service::CoverDropService, kube_client::KubeClient};
use std::path::Path;

pub async fn public_keys(kubeconfig_path: Option<impl AsRef<Path>>) -> anyhow::Result<()> {
    let kubeconfig_path = kubeconfig_path.map(|path| path.as_ref().to_path_buf());
    let kube_client = KubeClient::new(&kubeconfig_path).await?;

    let json = kube_client
        .forward_http_get_request(CoverDropService::IdentityApi, "/v1/public-keys")
        .await?;

    println!("{}", serde_json::to_string_pretty(&json)?);

    Ok(())
}
