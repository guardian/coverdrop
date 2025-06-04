use crate::{
    clients::{handle_response_json, new_reqwest_client},
    healthcheck::HealthCheck,
};
use reqwest::Url;

use super::{
    forms::post_rotate_covernode_id::RotateCoverNodeIdPublicKeyForm,
    models::UntrustedCoverNodeIdPublicKeyWithEpoch,
};

#[derive(Clone, Debug)]
pub struct IdentityApiClient {
    client: reqwest::Client,
    base_url: Url,
}

impl IdentityApiClient {
    pub fn new(base_url: Url) -> Self {
        let client = new_reqwest_client();
        Self { client, base_url }
    }

    /// GET    /v1/healthcheck
    pub async fn get_health_check(&self) -> anyhow::Result<HealthCheck> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .unwrap()
            .push("v1")
            .push("healthcheck");

        let health_check = self.client.get(url).send().await?;

        let health_check = handle_response_json(health_check).await?;

        Ok(health_check)
    }

    /// POST    /v1/public-keys/covernodes/me/rotate-id-key
    pub async fn post_rotate_covernode_id_key(
        &self,
        body: RotateCoverNodeIdPublicKeyForm,
    ) -> anyhow::Result<UntrustedCoverNodeIdPublicKeyWithEpoch> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .unwrap()
            .push("v1")
            .push("public-keys")
            .push("covernode")
            .push("me")
            .push("rotate-id-key");

        let new_signed_covernode_id_pk = self.client.post(url).json(&body).send().await?;

        let new_signed_covernode_id_pk = handle_response_json(new_signed_covernode_id_pk).await?;

        Ok(new_signed_covernode_id_pk)
    }
}
