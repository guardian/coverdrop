use chrono::{DateTime, Utc};
use reqwest::Url;

use crate::api::forms::{
    GetBackupDataForm, PatchJournalistStatusForm, PostBackupDataForm, PostBackupIdKeyForm,
    PostBackupMsgKeyForm,
};
use crate::api::models::dead_drops::{
    UnpublishedJournalistToUserDeadDrop, UnverifiedJournalistToUserDeadDropsList,
    UnverifiedUserToJournalistDeadDropsList,
};

use crate::client::JournalistStatus;
use crate::crypto::keys::public_key::PublicKey;
use crate::epoch::Epoch;
use crate::healthcheck::HealthCheck;
use crate::identity_api::models::UntrustedJournalistIdPublicKeyWithEpoch;
use crate::protocol::backup_data::BackupDataWithSignature;
use crate::protocol::keys::{
    CoverNodeIdKeyPair, CoverNodeIdPublicKey, CoverNodeMessagingPublicKey,
    CoverNodeProvisioningKeyPair, JournalistIdKeyPair, JournalistMessagingPublicKey,
    JournalistProvisioningKeyPair, UnregisteredJournalistIdPublicKey,
};

use crate::api::models::{
    dead_drops::{DeadDropId, UnpublishedUserToJournalistDeadDrop},
    realms::Realm,
};
use crate::clients::{handle_response, handle_response_json, new_reqwest_client};
use crate::system::keys::AdminKeyPair;

use super::forms::{
    DeleteJournalistForm, PatchJournalistForm, PostAdminPublicKeyForm,
    PostCoverNodeIdPublicKeyForm, PostCoverNodeMessagingPublicKeyForm,
    PostCoverNodeProvisioningPublicKeyForm, PostJournalistIdPublicKeyForm,
    PostJournalistMessagingPublicKeyForm, PostJournalistToCoverNodeMessageForm,
    PostSystemStatusEventForm, RotateJournalistIdPublicKeyFormForm,
};
use super::forms::{PostJournalistForm, PostJournalistProvisioningPublicKeyForm};
use super::models::covernode_id::CoverNodeIdentity;
use super::models::dead_drop_summary::DeadDropSummary;
use super::models::dead_drops::UnverifiedUserToJournalistDeadDrop;
use super::models::general::{PublishedStatusEvent, StatusEvent};
use super::models::journalist_id::JournalistIdentity;
use super::models::journalist_id_and_id_pk_rotation_form::JournalistIdAndPublicKeyRotationForm;
use super::models::messages::journalist_to_covernode_message::EncryptedJournalistToCoverNodeMessage;
use super::models::untrusted_keys_and_journalist_profiles::UntrustedKeysAndJournalistProfiles;

#[derive(Clone)]
pub struct ApiClient {
    client: reqwest::Client,
    base_url: Url,
}

impl ApiClient {
    pub fn new(base_url: Url) -> Self {
        let client = new_reqwest_client();
        Self { client, base_url }
    }

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

    pub async fn get_public_keys(&self) -> anyhow::Result<UntrustedKeysAndJournalistProfiles> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .unwrap()
            .push("v1")
            .push("public-keys");

        let keys = self.client.get(url).send().await?;

        let keys = handle_response_json(keys).await?;

        Ok(keys)
    }

    pub async fn post_backup_data(&self, form: PostBackupDataForm) -> anyhow::Result<()> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .unwrap()
            .push("v1")
            .push("backups")
            .push("data");

        let resp = self.client.post(url).json(&form).send().await?;

        handle_response(resp).await
    }

    pub async fn get_backup_data(
        &self,
        form: GetBackupDataForm,
    ) -> anyhow::Result<BackupDataWithSignature> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .unwrap()
            .push("v1")
            .push("backups")
            .push("data");

        let resp = self.client.get(url).json(&form).send().await?;

        let backup_data = handle_response_json(resp).await?;

        Ok(backup_data)
    }

    pub async fn post_backup_signing_pk(&self, form: PostBackupIdKeyForm) -> anyhow::Result<()> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .unwrap()
            .push("v1")
            .push("backups")
            .push("signing-public-key");

        let resp = self.client.post(url).json(&form).send().await?;

        handle_response(resp).await
    }

    pub async fn post_backup_encryption_pk(
        &self,
        form: PostBackupMsgKeyForm,
    ) -> anyhow::Result<()> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .unwrap()
            .push("v1")
            .push("backups")
            .push("encryption-public-key");

        let resp = self.client.post(url).json(&form).send().await?;

        handle_response(resp).await
    }

    pub async fn post_covernode_provisioning_pk(
        &self,
        form: PostCoverNodeProvisioningPublicKeyForm,
    ) -> anyhow::Result<()> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .unwrap()
            .push("v1")
            .push("public-keys")
            .push("covernode")
            .push("provisioning-public-key");

        let resp = self.client.post(url).json(&form).send().await?;

        handle_response(resp).await
    }

    pub async fn post_covernode_id_pk(
        &self,
        covernode_id: &CoverNodeIdentity,
        covernode_id_pk: &CoverNodeIdPublicKey,
        covernode_provisioning_key_pair: &CoverNodeProvisioningKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Epoch> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .unwrap()
            .push("v1")
            .push("public-keys")
            .push("covernode")
            .push("identity-public-key");

        let form = PostCoverNodeIdPublicKeyForm::new(
            covernode_id.clone(),
            covernode_id_pk.to_untrusted(),
            covernode_provisioning_key_pair,
            now,
        )?;

        let resp = self.client.post(url).json(&form).send().await?;

        let epoch = handle_response_json(resp).await?;

        Ok(epoch)
    }

    /// POST a covernode id public key to the API.
    /// This accepts a whole form as it is not meant to be done interactively
    /// while the secret key is present.
    pub async fn post_covernode_id_pk_form(
        &self,
        form: &PostCoverNodeIdPublicKeyForm,
    ) -> anyhow::Result<Epoch> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .unwrap()
            .push("v1")
            .push("public-keys")
            .push("covernode")
            .push("identity-public-key");

        let resp = self.client.post(url).json(form).send().await?;

        let epoch = handle_response_json(resp).await?;

        Ok(epoch)
    }

    pub async fn post_covernode_msg_pk(
        &self,
        covernode_msg_pk: &CoverNodeMessagingPublicKey,
        covernode_id_key_pair: &CoverNodeIdKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Epoch> {
        let form = PostCoverNodeMessagingPublicKeyForm::new(
            covernode_msg_pk.to_untrusted(),
            covernode_id_key_pair,
            now,
        )?;

        self.post_covernode_msg_pk_form(form).await
    }

    pub async fn post_covernode_msg_pk_form(
        &self,
        form: PostCoverNodeMessagingPublicKeyForm,
    ) -> anyhow::Result<Epoch> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .unwrap()
            .push("v1")
            .push("public-keys")
            .push("covernode")
            .push("messaging-public-key");

        let resp = self.client.post(url).json(&form).send().await?;

        let epoch = handle_response_json(resp).await?;

        Ok(epoch)
    }

    /// POST a journalist provisioning public key to the API.
    /// This accepts a whole form as it is not supposed to be done interactively
    /// while the secret key is present.
    pub async fn post_journalist_provisioning_pk(
        &self,
        form: PostJournalistProvisioningPublicKeyForm,
    ) -> anyhow::Result<()> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .unwrap()
            .push("v1")
            .push("public-keys")
            .push("journalists")
            .push("provisioning-public-key");

        let resp = self.client.post(url).json(&form).send().await?;

        handle_response(resp).await
    }

    pub async fn get_latest_status(&self) -> anyhow::Result<PublishedStatusEvent> {
        let mut url = self.base_url.clone();
        url.path_segments_mut().unwrap().push("v1").push("status");

        let status = self.client.get(url).send().await?;

        let status = handle_response_json(status).await?;

        Ok(status)
    }

    pub async fn post_admin_pk(&self, form: PostAdminPublicKeyForm) -> anyhow::Result<()> {
        let mut url = self.base_url.clone();

        url.path_segments_mut()
            .unwrap()
            .push("v1")
            .push("status")
            .push("public-key");

        let resp = self.client.post(url).json(&form).send().await?;

        handle_response(resp).await
    }

    pub async fn post_status_event(
        &self,
        status: StatusEvent,
        admin_key_pair: &AdminKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let mut url = self.base_url.clone();
        url.path_segments_mut().unwrap().push("v1").push("status");

        let body = PostSystemStatusEventForm::new(status, admin_key_pair, now)?;

        let resp = self.client.post(url).json(&body).send().await?;

        handle_response(resp).await
    }

    pub async fn post_status_event_form(
        &self,
        body: PostSystemStatusEventForm,
    ) -> anyhow::Result<()> {
        let mut url = self.base_url.clone();
        url.path_segments_mut().unwrap().push("v1").push("status");

        let resp = self.client.post(url).json(&body).send().await?;

        handle_response(resp).await
    }

    pub async fn post_journalist_form(&self, form: PostJournalistForm) -> anyhow::Result<()> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .unwrap()
            .push("v1")
            .push("public-keys")
            .push("journalists");

        let resp = self.client.post(url).json(&form).send().await?;

        handle_response(resp).await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn patch_journalist(
        &self,
        journalist_id: JournalistIdentity,
        display_name: Option<String>,
        sort_name: Option<String>,
        is_desk: Option<bool>,
        description: Option<String>,
        journalist_provisioning_key_pair: &JournalistProvisioningKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .unwrap()
            .push("v1")
            .push("public-keys")
            .push("journalists")
            .push("update-profile");

        let body = PatchJournalistForm::new(
            journalist_id,
            display_name,
            sort_name,
            is_desk,
            description,
            journalist_provisioning_key_pair,
            now,
        )?;

        let resp = self.client.patch(url).json(&body).send().await?;

        handle_response(resp).await
    }

    pub async fn patch_journalist_status(
        &self,
        journalist_id: JournalistIdentity,
        journalist_id_key_pair: &JournalistIdKeyPair,
        journalist_status: JournalistStatus,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .unwrap()
            .push("v1")
            .push("public-keys")
            .push("journalists")
            .push("update-status");

        let body = PatchJournalistStatusForm::new(
            journalist_id,
            journalist_status,
            journalist_id_key_pair,
            now,
        )?;

        let resp = self.client.patch(url).json(&body).send().await?;

        handle_response(resp).await
    }

    pub async fn post_journalist_id_pk_form(
        &self,
        form: PostJournalistIdPublicKeyForm,
    ) -> anyhow::Result<Epoch> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .unwrap()
            .push("v1")
            .push("public-keys")
            .push("journalists")
            .push("identity-public-key");

        let resp = self.client.post(url).json(&form).send().await?;

        let epoch = handle_response_json(resp).await?;

        Ok(epoch)
    }

    pub async fn get_journalist_id_pk_forms(
        &self,
    ) -> anyhow::Result<Vec<JournalistIdAndPublicKeyRotationForm>> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .unwrap()
            .push("v1")
            .push("public-keys")
            .push("journalists")
            .push("identity-public-key-form");

        let resp = self.client.get(url).send().await?;

        let forms = handle_response_json(resp).await?;

        Ok(forms)
    }

    pub async fn post_rotate_journalist_id_pk_form(
        &self,
        form: RotateJournalistIdPublicKeyFormForm,
    ) -> anyhow::Result<()> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .unwrap()
            .push("v1")
            .push("public-keys")
            .push("journalists")
            .push("identity-public-key-form");

        let resp = self.client.post(url).json(&form).send().await?;

        handle_response(resp).await
    }

    pub async fn get_journalist_id_pk_with_epoch(
        &self,
        candidate_journalist_id_pk: &UnregisteredJournalistIdPublicKey,
    ) -> anyhow::Result<Option<UntrustedJournalistIdPublicKeyWithEpoch>> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .unwrap()
            .push("v1")
            .push("public-keys")
            .push("journalists")
            .push("identity-public-key")
            .push(&candidate_journalist_id_pk.public_key_hex());

        let resp = self.client.get(url).send().await?;

        let epoch = handle_response_json(resp).await?;

        Ok(epoch)
    }

    pub async fn post_journalist_msg_pk(
        &self,
        msg_pk: &JournalistMessagingPublicKey,
        id_key_pair: &JournalistIdKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Epoch> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .unwrap()
            .push("v1")
            .push("public-keys")
            .push("journalists")
            .push("messaging-public-key");

        let form =
            PostJournalistMessagingPublicKeyForm::new(msg_pk.to_untrusted(), id_key_pair, now)?;

        let resp = self.client.post(url).json(&form).send().await?;

        let epoch = handle_response_json(resp).await?;

        Ok(epoch)
    }

    pub async fn post_journalist_msg(
        &self,
        j2c_msg: EncryptedJournalistToCoverNodeMessage,
        id_key_pair: &JournalistIdKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .unwrap()
            .push("v1")
            .push("journalist-messages");

        let form = PostJournalistToCoverNodeMessageForm::new(j2c_msg, id_key_pair, now)?;

        let resp = self.client.post(url).json(&form).send().await?;

        handle_response(resp).await
    }

    pub async fn delete_journalist(&self, form: &DeleteJournalistForm) -> anyhow::Result<()> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .unwrap()
            .push("v1")
            .push("public-keys")
            .push("journalists")
            .push("delete");

        let resp = self.client.delete(url).json(&form).send().await?;

        handle_response(resp).await
    }

    //
    // GET /v1/{realm}/dead_drops
    //

    pub async fn pull_user_dead_drops(
        &self,
        ids_greater_than: DeadDropId,
    ) -> anyhow::Result<UnverifiedJournalistToUserDeadDropsList> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .unwrap()
            .push("v1")
            .push((&Realm::User).into())
            .push("dead-drops");
        url.query_pairs_mut()
            .append_pair("ids_greater_than", &ids_greater_than.to_string());

        let user_dead_drops = self.client.get(url).send().await?;

        let user_dead_drops = handle_response_json(user_dead_drops).await?;

        Ok(user_dead_drops)
    }

    pub async fn pull_journalist_dead_drops(
        &self,
        ids_greater_than: DeadDropId,
        limit: Option<u32>,
    ) -> anyhow::Result<UnverifiedUserToJournalistDeadDropsList> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .unwrap()
            .push("v1")
            .push((&Realm::Journalist).into())
            .push("dead-drops");
        url.query_pairs_mut()
            .append_pair("ids_greater_than", &ids_greater_than.to_string());

        if let Some(limit) = limit {
            url.query_pairs_mut()
                .append_pair("limit", &limit.to_string());
        }

        let journalist_dead_drops = self.client.get(url).send().await?;

        let journalist_dead_drops = handle_response_json(journalist_dead_drops).await?;

        Ok(journalist_dead_drops)
    }

    pub async fn get_journalist_recent_dead_drop_summary(
        &self,
    ) -> anyhow::Result<Vec<DeadDropSummary>> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .unwrap()
            .push("v1")
            .push((&Realm::Journalist).into())
            .push("dead-drops")
            .push("recent-summary");

        let recent_dead_drops_summary = self.client.get(url).send().await?;

        let recent_dead_drops_summary = handle_response_json(recent_dead_drops_summary).await?;

        Ok(recent_dead_drops_summary)
    }

    pub async fn pull_all_journalist_dead_drops(
        &self,
        max_dead_drop_id: i32,
    ) -> anyhow::Result<UnverifiedUserToJournalistDeadDropsList> {
        let mut dead_drop_list: Vec<UnverifiedUserToJournalistDeadDrop> = Vec::new();
        let mut max_dead_drop_id = max_dead_drop_id;

        loop {
            let new_dead_drop_list = self
                .pull_journalist_dead_drops(max_dead_drop_id, None)
                .await?;

            if new_dead_drop_list.dead_drops.is_empty() {
                break;
            }

            max_dead_drop_id = new_dead_drop_list
                .dead_drops
                .iter()
                .max_by_key(|d| d.id)
                .map(|d| d.id)
                .unwrap_or(max_dead_drop_id);

            dead_drop_list.extend(new_dead_drop_list.dead_drops);
        }

        Ok(UnverifiedUserToJournalistDeadDropsList::new(dead_drop_list))
    }

    //
    // POST /v1/{realm}/dead-drops
    //

    pub async fn post_user_dead_drop(
        &self,
        signed_dead_drop: &UnpublishedJournalistToUserDeadDrop,
    ) -> anyhow::Result<()> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .unwrap()
            .push("v1")
            .push((&Realm::User).into())
            .push("dead-drops");

        let resp = self.client.post(url).json(signed_dead_drop).send().await?;

        handle_response(resp).await
    }

    pub async fn post_journalist_dead_drop(
        &self,
        dead_drop: &UnpublishedUserToJournalistDeadDrop,
    ) -> anyhow::Result<()> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .unwrap()
            .push("v1")
            .push((&Realm::Journalist).into())
            .push("dead-drops");

        let resp = self.client.post(url).json(dead_drop).send().await?;

        handle_response(resp).await
    }
}

impl std::fmt::Debug for ApiClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ApiClient({})", self.base_url)
    }
}
