use crate::{
    coverdrop_service::CoverDropService,
    dev::build::{cargo::cargo_metadata, docker::docker_build_rust},
    log_handler::LogHandler,
};

pub async fn build(service: CoverDropService) -> anyhow::Result<()> {
    let metadata = cargo_metadata().await?;

    let image_and_tag =
        docker_build_rust(&metadata.workspace_root, &service, &LogHandler::None).await?;

    tracing::info!("Built {} docker image", image_and_tag);

    Ok(())
}
