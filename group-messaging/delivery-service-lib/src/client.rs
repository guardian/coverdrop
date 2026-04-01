use common::{
    api::models::journalist_id::JournalistIdentity,
    clients::{handle_response, handle_response_json, new_reqwest_client},
};
use reqwest::Url;

use crate::{
    forms::{
        AddMembersForm, ConsumeKeyPackageForm, GetClientsForm, PublishKeyPackagesForm,
        ReceiveMessagesForm, RegisterClientForm, SendMessageForm,
    },
    models::GroupMessage,
};
use openmls::prelude::KeyPackageIn;

#[derive(Clone)]
pub struct DeliveryServiceClient {
    pub client: reqwest::Client,
    pub base_url: Url,
}

impl DeliveryServiceClient {
    pub fn new(base_url: Url) -> Self {
        let client = new_reqwest_client();
        Self { client, base_url }
    }

    /// Register a client with the delivery service
    pub async fn register_client(&self, form: RegisterClientForm) -> anyhow::Result<()> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .unwrap()
            .push("v1")
            .push("clients")
            .push("register");

        let resp = self.client.post(url).json(&form).send().await?;

        handle_response(resp).await
    }

    /// Publish additional key packages to the delivery service
    pub async fn publish_key_packages(&self, form: PublishKeyPackagesForm) -> anyhow::Result<()> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .unwrap()
            .push("v1")
            .push("clients")
            .push("key_package")
            .push("publish");

        let resp = self.client.post(url).json(&form).send().await?;

        handle_response(resp).await
    }

    /// Get the list of registered clients
    pub async fn get_clients(
        &self,
        form: GetClientsForm,
    ) -> anyhow::Result<Vec<JournalistIdentity>> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .unwrap()
            .push("v1")
            .push("clients")
            .push("list");

        let resp = self.client.post(url).json(&form).send().await?;

        let clients = handle_response_json(resp).await?;

        Ok(clients)
    }

    /// Consume a key package for another client
    pub async fn consume_key_package(
        &self,
        form: ConsumeKeyPackageForm,
    ) -> anyhow::Result<KeyPackageIn> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .unwrap()
            .push("v1")
            .push("clients")
            .push("key_package")
            .push("consume");

        let resp = self.client.post(url).json(&form).send().await?;

        let key_package = handle_response_json(resp).await?;

        Ok(key_package)
    }

    /// Add members to a group (sends both welcome and commit messages atomically)
    pub async fn add_members(&self, form: AddMembersForm) -> anyhow::Result<()> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .unwrap()
            .push("v1")
            .push("group")
            .push("add_members");

        let resp = self.client.post(url).json(&form).send().await?;

        handle_response(resp).await
    }

    /// Send a group message
    pub async fn send_message(&self, form: SendMessageForm) -> anyhow::Result<()> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .unwrap()
            .push("v1")
            .push("send")
            .push("message");

        let resp = self.client.post(url).json(&form).send().await?;

        handle_response(resp).await
    }

    /// Receive messages for a client
    pub async fn receive_messages(
        &self,
        form: ReceiveMessagesForm,
    ) -> anyhow::Result<Vec<GroupMessage>> {
        let mut url = self.base_url.clone();
        url.path_segments_mut().unwrap().push("v1").push("receive");

        let resp = self.client.post(url).json(&form).send().await?;

        let response = handle_response_json(resp).await?;

        Ok(response)
    }
}
