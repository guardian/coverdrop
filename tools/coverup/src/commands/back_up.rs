use std::path::Path;

use crate::{coverdrop_service::CoverDropService, data_copier_pod::DataCopierPod};

pub async fn back_up(
    output_directory: impl AsRef<Path>,
    kubeconfig_path: Option<impl AsRef<Path>>,
) -> anyhow::Result<()> {
    let output_directory = output_directory.as_ref();
    let kubeconfig_path = kubeconfig_path.map(|path| path.as_ref().to_path_buf());

    let covernode_data_copier_pod = DataCopierPod::get_or_create_in_background(
        CoverDropService::CoverNode.as_pvc_name(),
        kubeconfig_path.clone(),
    );
    let identity_api_data_copier_pod = DataCopierPod::get_or_create_in_background(
        CoverDropService::IdentityApi.as_pvc_name(),
        kubeconfig_path.clone(),
    );

    let covernode_data_copier_pod = covernode_data_copier_pod.await.await??;
    covernode_data_copier_pod
        .copy_from_pod(
            output_directory.join(CoverDropService::CoverNode.as_str()),
            "/",
        )
        .await?;

    let identity_api_data_copier_pod = identity_api_data_copier_pod.await.await??;
    identity_api_data_copier_pod
        .copy_from_pod(
            output_directory.join(CoverDropService::IdentityApi.as_str()),
            "/",
        )
        .await?;

    Ok(())
}
