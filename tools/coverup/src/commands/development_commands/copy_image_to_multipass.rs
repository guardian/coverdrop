use std::str::FromStr as _;

use rayon::prelude::*;

use crate::{
    docker::ImageAndTag,
    multipass::{copy_docker_image_to_nodes, list_coverdrop_nodes},
};

pub fn copy_all_images_to_multipass() -> anyhow::Result<()> {
    let nodes = list_coverdrop_nodes()?;

    let images = [
        ImageAndTag::from_str("coverdrop_api:dev").unwrap(),
        ImageAndTag::from_str("coverdrop_u2j-appender:dev").unwrap(),
        ImageAndTag::from_str("coverdrop_kinesis:dev").unwrap(),
        ImageAndTag::from_str("coverdrop_covernode:dev").unwrap(),
        ImageAndTag::from_str("coverdrop_identity-api:dev").unwrap(),
    ];

    images
        .par_iter()
        .map(|docker_image_and_tag| copy_docker_image_to_nodes(docker_image_and_tag, &nodes))
        .collect::<anyhow::Result<()>>()?;

    Ok(())
}

pub fn copy_image_to_multipass(images: &[ImageAndTag]) -> anyhow::Result<()> {
    let nodes = list_coverdrop_nodes()?;

    tracing::info!("Copying {:?} to {:?}", images, nodes);

    images
        .par_iter()
        .map(|docker_image_and_tag| copy_docker_image_to_nodes(docker_image_and_tag, &nodes))
        .collect::<anyhow::Result<()>>()?;

    Ok(())
}
