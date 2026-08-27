use crate::models::{
    Group, GroupIdExt, GroupMessageExt, MlsMessageContent, MlsMessageContentWithId,
    SentinelIdentityExt, SentinelIdentityWithLeafIndex,
};
use anyhow::Context;
use chrono::{DateTime, Utc};
use common::api::models::sentinel_id::SentinelIdentity;
use common::crypto::keys::signing::traits::PublicSigningKey;
use common::protocol::keys::{OrganizationPublicKeyFamilyList, SentinelIdKeyPair};
use common::time;
use delivery_service_lib::client::DeliveryServiceClient;
use delivery_service_lib::forms::{
    AddMembersForm, ConsumeKeyPackageForm, GetClientsForm, PublishKeyPackagesForm,
    ReceiveMessagesForm, RegisterClientForm, SendMessageForm,
};
use delivery_service_lib::models::KeyPackageWithClientId;
use delivery_service_lib::tls_serialized::TlsSerialized;
use delivery_service_lib::{MLS_CIPHERSUITE, PROTOCOL_VERSION};
use journalist_vault::JournalistVault;
use journalist_vault::{GroupId, GroupMessage, GroupMessageContent, MessageId};
use openmls::prelude::*;
use openmls_basic_credential::SignatureKeyPair;
use openmls_rust_crypto::RustCrypto;
use openmls_sqlx_storage::SqliteStorageProvider;
use openmls_traits::OpenMlsProvider;
use reqwest::Url;
use tokio::sync::Mutex;

use crate::vault_provider::{CborCodec, VaultProvider};

/// Thread-safe wrapper around the inner GroupMessagingService.
/// All operations are serialized through an internal mutex to prevent concurrent
/// access to MLS group state, which would cause SecretReuseErrors.
pub struct GroupMessagingService {
    inner: Mutex<GroupMessagingServiceInner>,
}

impl GroupMessagingService {
    /// Create a new group messaging service given a vault for persistent storage and a delivery service URL.
    /// Returns an error if the vault doesn't contain a Sentinel Id and key pair.
    pub async fn new(
        delivery_service_url: Url,
        vault: &JournalistVault,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        let inner = GroupMessagingServiceInner::new(delivery_service_url, vault, now).await?;
        Ok(Self {
            inner: Mutex::new(inner),
        })
    }

    pub async fn client_id(&self) -> SentinelIdentity {
        self.inner.lock().await.client_id()
    }

    pub async fn register(&self, num_key_packages: usize) -> anyhow::Result<()> {
        self.inner.lock().await.register(num_key_packages).await
    }

    pub async fn publish_key_packages(&self, num_key_packages: usize) -> anyhow::Result<()> {
        self.inner
            .lock()
            .await
            .publish_key_packages(num_key_packages)
            .await
    }

    pub async fn get_clients(&self) -> anyhow::Result<Vec<SentinelIdentity>> {
        self.inner.lock().await.get_clients().await
    }

    pub async fn create_group_with_members(
        &self,
        group_id: GroupId,
        group_members: Vec<SentinelIdentity>,
        display_name: &str,
        description: &str,
        public_keys: &OrganizationPublicKeyFamilyList,
    ) -> anyhow::Result<()> {
        self.inner
            .lock()
            .await
            .create_group_with_members(
                group_id,
                group_members,
                display_name,
                description,
                public_keys,
            )
            .await
    }

    pub async fn group_ids(&self) -> anyhow::Result<Vec<GroupId>> {
        self.inner.lock().await.group_ids().await
    }

    pub async fn modify_group(
        &self,
        group_id: &GroupId,
        new_group_membership: Vec<SentinelIdentity>,
        new_display_name: &String,
        new_description: &String,
        public_keys: &OrganizationPublicKeyFamilyList,
    ) -> anyhow::Result<()> {
        self.inner
            .lock()
            .await
            .modify_group(
                group_id,
                new_group_membership,
                new_display_name,
                new_description,
                public_keys,
            )
            .await
    }

    pub async fn send_message(
        &self,
        group_id: &GroupId,
        content_with_id: &MlsMessageContentWithId,
    ) -> anyhow::Result<()> {
        self.inner
            .lock()
            .await
            .send_message(group_id, content_with_id)
            .await
    }

    pub async fn receive_and_store_messages(
        &self,
        public_keys: &OrganizationPublicKeyFamilyList,
    ) -> anyhow::Result<Vec<GroupMessage>> {
        self.inner
            .lock()
            .await
            .receive_and_store_messages(public_keys)
            .await
    }

    pub async fn rotate_signature_key(
        &self,
        new_id_key_pair: SentinelIdKeyPair,
    ) -> anyhow::Result<()> {
        self.inner
            .lock()
            .await
            .rotate_signature_key(new_id_key_pair)
            .await
    }

    #[cfg(feature = "integration-tests")]
    pub async fn has_active_mls_group(&self, group_id: &GroupId) -> anyhow::Result<bool> {
        self.inner.lock().await.has_active_mls_group(group_id).await
    }

    pub async fn get_group_with_messages(&self, group_id: GroupId) -> anyhow::Result<Group> {
        self.inner
            .lock()
            .await
            .get_group_with_messages(group_id)
            .await
    }

    pub async fn get_groups_and_messages(&self) -> anyhow::Result<Vec<Group>> {
        self.inner.lock().await.get_groups_and_messages().await
    }

    pub async fn update_group_messages_read_status(
        &self,
        group_id: &GroupId,
        messages: Vec<MessageId>,
        read: bool,
    ) -> anyhow::Result<()> {
        self.inner
            .lock()
            .await
            .update_group_messages_read_status(group_id, messages, read)
            .await
    }
}

/// Internal implementation of MLS group messaging operations.
/// All access must be serialized via the Mutex in `GroupMessagingService`.
struct GroupMessagingServiceInner {
    vault: JournalistVault,
    delivery_service_client: DeliveryServiceClient,
    // The following fields are either stored in the vault directly or derived from data
    // in the vault, but are cached here to avoid having to query the vault for them every time we need them.
    client_id: SentinelIdentity,
    client_id_key: SentinelIdKeyPair,
    // MLS type corresponding to client_id_key
    signer: SignatureKeyPair,
    // MLS type corresponding to client_id and client_id_key
    credential_with_key: CredentialWithKey,
}

impl GroupMessagingServiceInner {
    /// Create a new group messaging service given a vault for persistent storage and a delivery service URL.
    /// Returns an error if the vault doesn't contain a Sentinel Id and key pair.
    pub async fn new(
        delivery_service_url: Url,
        vault: &JournalistVault,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        // Run OpenMLS storage migrations on the vault database
        let mut conn = vault.pool.acquire().await?;
        let mut storage = SqliteStorageProvider::<CborCodec>::new(&mut conn);
        storage
            .run_migrations()
            .map_err(|e| anyhow::anyhow!("Failed to run OpenMLS storage migrations: {}", e))?;

        let client_id = vault
            .sentinel_id()
            .await?
            .context("Attempted to open a vault with no Sentinel Id")?;
        let client_id_key = vault
            .latest_sentinel_id_key_pair(now)
            .await?
            .context("Attempted to open a vault with no Sentinel Id key pair")?;

        let (credential_with_key, signer) = generate_credential_and_signature_key_pair(
            MLS_CIPHERSUITE,
            &client_id,
            client_id_key.clone(),
        );

        Ok(Self {
            vault: vault.clone(),
            delivery_service_client: DeliveryServiceClient::new(delivery_service_url),
            client_id: client_id.clone(),
            client_id_key,
            signer,
            credential_with_key,
        })
    }

    fn client_id(&self) -> SentinelIdentity {
        self.client_id.clone()
    }

    /// Generate key packages for this client and return them along with their hashes.
    /// The hash is used by the delivery service as a unique identifier for each key package.
    ///
    /// This intentionally uses a plain connection rather than a transaction so that
    /// generated key packages are persisted immediately. If a subsequent request to
    /// the delivery service fails, we can't be sure whether the DS received the key
    /// packages or not, so it's safer to keep them in local storage.
    async fn generate_key_packages(&self, count: usize) -> anyhow::Result<Vec<KeyPackageIn>> {
        let mut conn = self.vault.pool.acquire().await?;
        let provider = VaultProvider::new(&mut conn);

        (0..count)
            .map(|_| {
                let key_package_bundle = KeyPackage::builder()
                    .key_package_extensions(Extensions::empty())
                    .build(
                        MLS_CIPHERSUITE,
                        &provider,
                        &self.signer,
                        self.credential_with_key.clone(),
                    )
                    .map_err(|e| anyhow::anyhow!("Failed to build key package: {}", e))?;

                let key_package = key_package_bundle.key_package();

                Ok(KeyPackageIn::from(key_package.clone()))
            })
            .collect::<Result<Vec<_>, _>>()
    }

    /// Register this client with the delivery service
    pub async fn register(&self, num_key_packages: usize) -> anyhow::Result<()> {
        let key_packages = self.generate_key_packages(num_key_packages).await?;
        let form = RegisterClientForm::new(key_packages, &self.client_id_key, time::now())?;
        self.delivery_service_client.register_client(form).await
    }

    /// Publish additional key packages to the delivery service
    pub async fn publish_key_packages(&self, num_key_packages: usize) -> anyhow::Result<()> {
        let key_packages = self.generate_key_packages(num_key_packages).await?;
        let form = PublishKeyPackagesForm::new(key_packages, &self.client_id_key, time::now())?;
        self.delivery_service_client
            .publish_key_packages(form)
            .await
    }

    /// Get the list of registered clients
    pub async fn get_clients(&self) -> anyhow::Result<Vec<SentinelIdentity>> {
        let form = GetClientsForm::new(&self.client_id_key, time::now())?;

        self.delivery_service_client.get_clients(form).await
    }

    /// Get a key package for another client from the delivery service and verify its authenticity
    async fn get_key_package(
        &self,
        target_client_id: &SentinelIdentity,
        public_keys: &OrganizationPublicKeyFamilyList,
    ) -> anyhow::Result<KeyPackageWithClientId> {
        let form =
            ConsumeKeyPackageForm::new(target_client_id.clone(), &self.client_id_key, time::now())?;

        let key_package = self
            .delivery_service_client
            .consume_key_package(form)
            .await?;

        // AUTH compare the key package credential and signing pk to the trusted public key hierarchy
        let key_package_credential_with_key = key_package.unverified_credential();
        authenticate_credential_and_key(
            key_package_credential_with_key.credential,
            &key_package_credential_with_key.signature_key,
            public_keys,
            Some(target_client_id),
        )?;

        // AUTH validate calls KeyPackage.verify which verifies the signature of the payload
        // using the signing public key in the key package's leaf node.
        let crypto = RustCrypto::default();
        let validated_key_package = key_package.validate(&crypto, PROTOCOL_VERSION)?;

        Ok(KeyPackageWithClientId {
            client_id: target_client_id.clone(),
            key_package: validated_key_package,
        })
    }

    /// Create a new group with the given display name, and description,
    /// then add the given members to the group and send them Welcome messages.
    /// If self is in the list of group members, it is removed since the group creator is automatically added to the group.
    pub async fn create_group_with_members(
        &self,
        group_id: GroupId,
        group_members: Vec<SentinelIdentity>,
        display_name: &str,
        description: &str,
        public_keys: &OrganizationPublicKeyFamilyList,
    ) -> anyhow::Result<()> {
        let mut tx = self.vault.pool.begin().await?;

        let mut group = self
            .create_or_load_group_in_transaction(
                &mut tx,
                group_id.clone(),
                display_name,
                description,
            )
            .await?;

        // Add a GroupInfo message to the vault. This is necessary so that this client has the same group name and
        // description history as other members of the group.
        self.vault
            .insert_j2j_message(
                &mut tx,
                &GroupMessage::new(
                    MessageId::new(),
                    self.client_id.clone(),
                    group_id.clone(),
                    GroupMessageContent::GroupInfo {
                        display_name: display_name.to_string(),
                        description: description.to_string(),
                    },
                    true, // own message should always be considered 'read'
                    time::now(),
                ),
            )
            .await?;

        // If self is in the list of group members, remove it since the group creator is automatically added to the group.
        let group_members: Vec<SentinelIdentity> = group_members
            .into_iter()
            .filter(|member| member != &self.client_id)
            .collect();

        self.add_members_to_group_in_transaction(
            &mut tx,
            &mut group,
            group_members,
            public_keys,
            true,
        )
        .await?;

        tx.commit().await?;

        Ok(())
    }

    /// Create a new MLS group. If a group with the same ID already exists, this is a no-op.
    async fn create_or_load_group_in_transaction(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        group_id: GroupId,
        display_name: &str,
        description: &str,
    ) -> anyhow::Result<MlsGroup> {
        // Check if the group already exists using the transaction connection
        let provider = VaultProvider::new(&mut *tx);
        if let Some(group) = MlsGroup::load(provider.storage(), &group_id.to_open_mls_group_id())? {
            tracing::info!(
                "Group with id {} already exists, skipping creation",
                group_id
            );
            // No changes to commit — just return the existing group
            return Ok(group);
        }

        let group_config = &MlsGroupCreateConfig::builder()
            .ciphersuite(MLS_CIPHERSUITE)
            .use_ratchet_tree_extension(true)
            .build();

        // Create the MLS group and insert the vault group row in the same transaction
        let group = MlsGroup::new_with_group_id(
            &provider,
            &self.signer,
            group_config,
            group_id.to_open_mls_group_id(),
            self.credential_with_key.clone(),
        )?;

        self.vault
            .insert_group(&mut *tx, &group_id, Some(display_name), Some(description))
            .await?;

        Ok(group)
    }

    pub async fn group_ids(&self) -> anyhow::Result<Vec<GroupId>> {
        self.vault
            .get_groups()
            .await?
            .into_iter()
            .map(|group_row| Ok(group_row.id))
            .collect()
    }

    /// Modifies a groups info (display name, description) and membership in a single operation
    /// in the following order: remove members, modify group info, add members.
    /// This ensures removed members don't receive group info change messages,
    /// and new members receive a single GroupInfo message rather than
    /// GroupNameChanged and GroupDescriptionChanged update messages.
    ///
    /// TODO this doesn't yet cover leaving a group i.e. when new_group_membership doesn't include self.
    /// https://github.com/guardian/coverdrop-internal/issues/4059
    pub async fn modify_group(
        &self,
        group_id: &GroupId,
        new_group_membership: Vec<SentinelIdentity>,
        new_display_name: &String,
        new_description: &String,
        public_keys: &OrganizationPublicKeyFamilyList,
    ) -> anyhow::Result<()> {
        let mut tx = self.vault.pool.begin().await?;

        let provider = VaultProvider::new(&mut tx);
        let mut group = MlsGroup::load(provider.storage(), &group_id.to_open_mls_group_id())?
            .ok_or_else(|| anyhow::anyhow!("Group not found"))?;

        // Figure out if any members are being added or removed.
        let current_group_members = self.get_group_members(&group, false)?;
        let current_group_member_ids = current_group_members
            .iter()
            .map(|id_with_leaf_index| id_with_leaf_index.identity.clone())
            .collect::<Vec<_>>();

        tracing::info!(
            "Modifying group membership for group {}. Current members: {:?}, new members: {:?}",
            group_id,
            current_group_member_ids,
            new_group_membership
        );

        let sentinel_ids_to_add = new_group_membership
            .iter()
            .filter(|member| !current_group_member_ids.contains(member))
            .cloned()
            .collect::<Vec<_>>();

        let sentinel_identities_with_leaf_node_indexes_to_remove = current_group_members
            .iter()
            .filter(|member| !new_group_membership.contains(&member.identity))
            .collect::<Vec<_>>();

        // 1. Remove members first, so they don't receive group info change messages.
        if !sentinel_identities_with_leaf_node_indexes_to_remove.is_empty() {
            self.remove_members_from_group_in_transaction(
                &mut tx,
                &mut group,
                sentinel_identities_with_leaf_node_indexes_to_remove,
            )
            .await?;
        }

        // 2. Modify group info, so new members will receive the updated info.
        self.modify_group_info_in_transaction(&mut tx, group_id, new_display_name, new_description)
            .await?;

        // 3. Add new members last, so they receive the updated group info.
        if !sentinel_ids_to_add.is_empty() {
            self.add_members_to_group_in_transaction(
                &mut tx,
                &mut group,
                sentinel_ids_to_add,
                public_keys,
                false,
            )
            .await?;
        }

        tx.commit().await?;

        Ok(())
    }

    /// Modify the display name and/or description of a group.
    /// If either has changed, sends a `GroupNameChanged` and/or `GroupDescriptionChanged`
    /// message to the group and updates the local vault.
    async fn modify_group_info_in_transaction(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        group_id: &GroupId,
        new_display_name: &String,
        new_description: &String,
    ) -> anyhow::Result<()> {
        let current = self
            .vault
            .get_group_with_connection(&mut *tx, group_id)
            .await?;
        let current_name = current.display_name.unwrap_or_default();
        let current_description = current.description.unwrap_or_default();

        let name_changed = new_display_name != &current_name;
        let description_changed = new_description != &current_description;

        if !name_changed && !description_changed {
            return Ok(());
        }

        if name_changed {
            let content = MlsMessageContentWithId::new_from_content(
                MlsMessageContent::GroupNameChanged(new_display_name.to_string()),
            );
            self.send_message_in_transaction(tx, group_id, &content)
                .await?;
        }

        if description_changed {
            let content = MlsMessageContentWithId::new_from_content(
                MlsMessageContent::GroupDescriptionChanged(new_description.clone()),
            );
            self.send_message_in_transaction(tx, group_id, &content)
                .await?;
        }

        self.vault
            .update_group(&mut *tx, group_id, new_display_name, new_description)
            .await?;

        Ok(())
    }

    async fn add_members_to_group_in_transaction(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        group: &mut MlsGroup,
        group_members: Vec<SentinelIdentity>,
        public_keys: &OrganizationPublicKeyFamilyList,
        group_creation: bool,
    ) -> anyhow::Result<()> {
        let group_id = GroupId::from_open_mls_group_id(group.group_id())?;
        let group_info = self
            .vault
            .get_group_with_connection(&mut *tx, &group_id)
            .await?;

        let provider = VaultProvider::new(&mut *tx);

        let mut key_packages_with_client_ids = Vec::new();
        // TODO fetch all key packages in a single request
        // https://github.com/guardian/coverdrop-internal/issues/3919
        for member in group_members {
            key_packages_with_client_ids.push(self.get_key_package(&member, public_keys).await?);
        }

        let (new_members, key_packages): (Vec<_>, Vec<_>) = key_packages_with_client_ids
            .into_iter()
            .map(|kp| (kp.client_id, kp.key_package))
            .unzip();

        let (mls_message_out, welcome, _) =
            group.add_members(&provider, &self.signer, &key_packages)?;

        // Get the list of existing members (everyone except the new members being added)
        let existing_members: Vec<SentinelIdentity> = self
            .get_group_members(group, true)?
            .into_iter()
            .map(|id_with_leaf_index| id_with_leaf_index.identity)
            .collect();

        // Serialize the welcome message
        let welcome_message = TlsSerialized::serialize(&welcome)?;

        // Serialize the commit message for existing members
        let commit_message = TlsSerialized::serialize(&mls_message_out)?;

        let form = AddMembersForm::new(
            welcome_message,
            commit_message,
            existing_members,
            new_members.clone(),
            &self.client_id_key,
            time::now(),
        )?;

        // TODO there is a consensus issue here if this request makes it to the DS but the response is dropped.
        // We should think about having clients send commits themselves, and also merging them upon receipt.
        // https://github.com/guardian/coverdrop-internal/issues/4054
        self.delivery_service_client.add_members(form).await?;

        group.merge_pending_commit(&provider)?;

        // Send a GroupInfo message only to new members so they receive the group name and description.
        // This can't be done as part of the AddMembersForm because this client should only merge the pending
        // commit after receiving a success response from the DS.
        let group_info_message =
            MlsMessageContentWithId::new_from_content(MlsMessageContent::GroupInfo {
                display_name: group_info.display_name.unwrap_or_default(),
                description: group_info.description.unwrap_or_default(),
            });
        let mls_message_out =
            group.create_message(&provider, &self.signer, &group_info_message.to_bytes()?)?;
        self.send_mls_message_to_recipients(mls_message_out, new_members.clone())
            .await?;

        // If this is an update to an existing group, then
        // store a UsersAdded message locally for the sender
        if !group_creation {
            self.vault
                .insert_j2j_message(
                    &mut *tx,
                    &GroupMessage::new(
                        MessageId::new(),
                        self.client_id.clone(),
                        group_id.clone(),
                        GroupMessageContent::UsersAdded(new_members),
                        true, // own message should always be considered 'read'
                        time::now(),
                    ),
                )
                .await?;
        }

        Ok(())
    }

    async fn remove_members_from_group_in_transaction(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        group: &mut MlsGroup,
        sentinel_identities_with_leaf_node_indexes_to_remove: Vec<&SentinelIdentityWithLeafIndex>,
    ) -> anyhow::Result<()> {
        let provider = VaultProvider::new(&mut *tx);

        let leaf_node_indexes_to_remove = sentinel_identities_with_leaf_node_indexes_to_remove
            .iter()
            .map(|id_with_leaf_index| id_with_leaf_index.leaf_index)
            .collect::<Vec<_>>();

        let sentinel_identities_to_remove = sentinel_identities_with_leaf_node_indexes_to_remove
            .iter()
            .map(|id_with_leaf_index| id_with_leaf_index.identity.clone())
            .collect::<Vec<_>>();

        let group_id = GroupId::from_open_mls_group_id(group.group_id())?;

        let (mls_message_out, _welcome_option, _group_info) =
            group.remove_members(&provider, &self.signer, &leaf_node_indexes_to_remove)?;

        let published_at = self
            .send_mls_message_to_group(group, mls_message_out)
            .await?;

        group.merge_pending_commit(&provider)?;

        // Store a UsersRemoved message locally for the sender
        self.vault
            .insert_j2j_message(
                &mut *tx,
                &GroupMessage::new(
                    MessageId::new(),
                    self.client_id.clone(),
                    group_id.clone(),
                    GroupMessageContent::UsersRemoved(sentinel_identities_to_remove),
                    true, // own message should always be considered 'read'
                    published_at,
                ),
            )
            .await?;

        Ok(())
    }

    /// Wraps get_group_members, returning the list of `SentinelIdentities` of the members of the group.
    /// Excludes self's client_id if `exclude_self` is true.
    fn get_group_member_ids(
        &self,
        group: &MlsGroup,
        exclude_self: bool,
    ) -> anyhow::Result<Vec<SentinelIdentity>> {
        let members = self
            .get_group_members(group, exclude_self)?
            .into_iter()
            .map(|id_with_leaf_index| id_with_leaf_index.identity)
            .collect();

        Ok(members)
    }

    /// returns the list of `SentinelIdentities` and `LeafNodeIndex`es of the members of the group.
    /// Excludes self's client_id if `exclude_self` is true.
    fn get_group_members(
        &self,
        group: &MlsGroup,
        exclude_self: bool,
    ) -> anyhow::Result<Vec<SentinelIdentityWithLeafIndex>> {
        let members = group
            .members()
            .filter_map(|member| {
                let identity =
                    SentinelIdentity::from_mls_credential(member.credential.clone()).ok()?;
                Some(SentinelIdentityWithLeafIndex {
                    identity,
                    leaf_index: member.index,
                })
            })
            // Maybe filter out the member who is adding (self)
            .filter(|id_with_leaf_index| {
                !exclude_self || id_with_leaf_index.identity != self.client_id
            })
            .collect();

        Ok(members)
    }

    /// Send an application message to the group and record it as a sent message in
    /// the vault.
    pub async fn send_message(
        &self,
        group_id: &GroupId,
        content_with_id: &MlsMessageContentWithId,
    ) -> anyhow::Result<()> {
        let mut tx = self.vault.pool.begin().await?;
        self.send_message_in_transaction(&mut tx, group_id, content_with_id)
            .await?;
        tx.commit().await?;
        Ok(())
    }

    /// Send an application message using an existing transaction.
    async fn send_message_in_transaction(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        group_id: &GroupId,
        content_with_id: &MlsMessageContentWithId,
    ) -> anyhow::Result<()> {
        let provider = VaultProvider::new(&mut *tx);

        let mut group = MlsGroup::load(provider.storage(), &group_id.to_open_mls_group_id())?
            .ok_or_else(|| anyhow::anyhow!("Group not found"))?;

        let mls_message_out =
            group.create_message(&provider, &self.signer, &content_with_id.to_bytes()?)?;
        let published_at = self
            .send_mls_message_to_group(&mut group, mls_message_out)
            .await?;

        self.vault
            .insert_j2j_message(
                &mut *tx,
                &GroupMessage::new(
                    content_with_id.id,
                    self.client_id.clone(),
                    group_id.clone(),
                    content_with_id.content.clone().into(),
                    true, // own message should always be considered 'read'
                    published_at,
                ),
            )
            .await?;

        Ok(())
    }

    /// Sends an `MlsMessageOut` to all members of the group, excluding self.
    /// Returns the `published_at` timestamp returned by the delivery service.
    async fn send_mls_message_to_group(
        &self,
        group: &mut MlsGroup,
        mls_message_out: MlsMessageOut,
    ) -> anyhow::Result<DateTime<Utc>> {
        let recipients = self.get_group_member_ids(group, true)?;
        self.send_mls_message_to_recipients(mls_message_out, recipients)
            .await
    }

    async fn send_mls_message_to_recipients(
        &self,
        mls_message_out: MlsMessageOut,
        recipients: Vec<SentinelIdentity>,
    ) -> anyhow::Result<DateTime<Utc>> {
        let message = TlsSerialized::serialize(&mls_message_out)?;
        let form = SendMessageForm::new(message, recipients, &self.client_id_key, time::now())?;
        let response = self.delivery_service_client.send_message(form).await?;
        Ok(response.published_at)
    }

    /// Receive, process, and store messages from the delivery service
    pub async fn receive_and_store_messages(
        &self,
        public_keys: &OrganizationPublicKeyFamilyList,
    ) -> anyhow::Result<Vec<GroupMessage>> {
        let current_max_message_id = self.vault.max_delivery_service_message_id().await?;
        let form =
            ReceiveMessagesForm::new(current_max_message_id, &self.client_id_key, time::now())?;

        let messages = self.delivery_service_client.receive_messages(form).await?;

        // TODO once a message is processed, it can't be decrypted again, and attempts to do so return
        // a SecretReuseError. Handle this as part of https://github.com/guardian/coverdrop-internal/issues/3918
        // by having the DS store the epoch of the key package / message, then not attempting to authenticate and
        // process key packages or messages using a stale key hierarchy.

        let mut decrypted_messages = Vec::new();

        for msg in messages {
            let mut tx = self.vault.pool.begin().await?;
            let provider = VaultProvider::new(&mut tx);

            // Deserialize the MLS message and extract its body
            let msg_bytes = &msg.content;
            let mls_message_in = msg_bytes.deserialize::<MlsMessageIn>()?;
            let mls_message_body = mls_message_in.extract();

            // TODO break into helper functions for processing different message types
            match mls_message_body {
                MlsMessageBodyIn::Welcome(welcome) => {
                    // It's important that the join config includes ratchet tree extension so that new additions
                    // to the group receive the public ratchet tree. Otherwise only the group creator can add clients.
                    let group_join_config = MlsGroupJoinConfig::builder()
                        .use_ratchet_tree_extension(true)
                        .build();
                    // We're skipping the ProcessedWelcome stage for the moment since we don't need
                    // to retrieve information from the `Welcome` about the ratchet tree and PSKs
                    let join_builder =
                        StagedWelcome::build_from_welcome(&provider, &group_join_config, welcome)?;

                    // If we were previously a member of this group (e.g. removed and re-added),
                    // replace the old MLS group state so the Welcome can create a fresh one.
                    let staged_welcome = join_builder.replace_old_group().build()?;

                    // AUTH authenticate the Welcome by verifying the credential and signature key in the welcome sender's leaf node
                    let welcome_sender = staged_welcome.welcome_sender()?;
                    authenticate_credential_and_key(
                        welcome_sender.credential().clone(),
                        welcome_sender.signature_key(),
                        public_keys,
                        None,
                    )?;

                    // AUTH inspect every leaf node of the ratchet tree to authenticate its
                    // credential and signature key.
                    // https://book.openmls.tech/user_manual/credential_validation.html
                    staged_welcome.members().try_for_each(|member| {
                        let credential = member.credential;
                        let signature_key = SignaturePublicKey::from(member.signature_key);

                        authenticate_credential_and_key(
                            credential,
                            &signature_key,
                            public_keys,
                            None,
                        )
                    })?;

                    let group = staged_welcome.into_group(&provider)?;

                    let group_id = GroupId::from_open_mls_group_id(group.group_id())?;

                    // Insert with empty name/description; the GroupCreated message
                    // that follows the Welcome will update these fields.
                    self.vault
                        .insert_group(&mut tx, &group_id, None, None)
                        .await?;
                }
                MlsMessageBodyIn::PrivateMessage(private_msg) => {
                    // All handshake and application messages use PrivateMessage (encrypted)
                    let protocol_message = ProtocolMessage::from(private_msg);

                    let mls_group_id = protocol_message.group_id();
                    let group_id = GroupId::from_open_mls_group_id(mls_group_id)?;
                    let Some(mut group) = MlsGroup::load(provider.storage(), mls_group_id)? else {
                        tracing::error!("Received message for unknown group: {:?}", mls_group_id);
                        continue;
                    };

                    // AUTH processing performs all syntactic and semantic validation checks and verifies the message's signature
                    // https://book.openmls.tech/user_manual/processing.html#processing-messages-in-groups
                    let processed = group.process_message(&provider, protocol_message)?;

                    let sender =
                        SentinelIdentity::from_mls_credential(processed.credential().clone())?;

                    match processed.into_content() {
                        ProcessedMessageContent::ApplicationMessage(app_msg) => {
                            let group_message = GroupMessage::from_mls_application_message(
                                app_msg,
                                group.group_id(),
                                sender,
                                msg.published_at,
                            )?;

                            decrypted_messages.push(group_message.clone());

                            self.vault
                                .insert_j2j_message(&mut tx, &group_message)
                                .await?;

                            // Make updates to the group depending on the content of the message
                            match &group_message.content {
                                GroupMessageContent::GroupInfo {
                                    display_name,
                                    description,
                                } => {
                                    tracing::info!(
                                        "Received GroupInfo for group {}",
                                        group_message.group_id
                                    );
                                    self.vault
                                        .update_group(
                                            &mut tx,
                                            &group_message.group_id,
                                            display_name,
                                            description,
                                        )
                                        .await?;
                                }
                                GroupMessageContent::GroupNameChanged(name) => {
                                    tracing::info!(
                                        "Received GroupNameChanged for group {}",
                                        group_message.group_id
                                    );
                                    self.vault
                                        .update_group_name(&mut tx, &group_message.group_id, name)
                                        .await?;
                                }
                                GroupMessageContent::GroupDescriptionChanged(description) => {
                                    tracing::info!(
                                        "Received GroupDescriptionChanged for group {}",
                                        group_message.group_id
                                    );
                                    self.vault
                                        .update_group_description(
                                            &mut tx,
                                            &group_message.group_id,
                                            description,
                                        )
                                        .await?;
                                }
                                _ => {}
                            }
                        }
                        ProcessedMessageContent::ProposalMessage(_) => {
                            // Handle proposals if needed
                        }
                        ProcessedMessageContent::ExternalJoinProposalMessage(_) => {
                            // Handle external join proposals if needed
                        }
                        ProcessedMessageContent::StagedCommitMessage(staged_commit) => {
                            let staged_commit = *staged_commit;

                            let sender_update_path_leaf_node =
                                staged_commit.update_path_leaf_node();
                            if let Some(leaf_node) = sender_update_path_leaf_node {
                                // AUTH authenticate the sender of the commit by verifying the credential and signature key in the update path leaf node
                                authenticate_credential_and_key(
                                    leaf_node.credential().clone(),
                                    leaf_node.signature_key(),
                                    public_keys,
                                    None,
                                )?;
                            }

                            // AUTH authenticate add proposals
                            let mut added_members = Vec::new();
                            for queued_add_proposal in staged_commit.add_proposals() {
                                let new_member_key_package =
                                    queued_add_proposal.add_proposal().key_package();
                                let leaf_node = new_member_key_package.leaf_node();
                                let credential = leaf_node.credential().clone();
                                let signature_key = leaf_node.signature_key();

                                authenticate_credential_and_key(
                                    credential.clone(),
                                    signature_key,
                                    public_keys,
                                    None,
                                )?;

                                let added_id = SentinelIdentity::from_mls_credential(credential)?;
                                added_members.push(added_id);
                            }

                            // AUTH authenticate update proposals.
                            // NOTE if credential verification is expensive, we can check whether
                            // the signature key and credential have actually changed before authenticating
                            for queued_update_proposal in staged_commit.update_proposals() {
                                let leaf_node =
                                    queued_update_proposal.update_proposal().leaf_node();
                                let credential = leaf_node.credential().clone();
                                let signature_key = leaf_node.signature_key();

                                authenticate_credential_and_key(
                                    credential,
                                    signature_key,
                                    public_keys,
                                    None,
                                )?;
                            }

                            // Handle remove proposals
                            let mut removed_members = Vec::new();
                            for queued_remove_proposal in staged_commit.remove_proposals() {
                                let removed_index =
                                    queued_remove_proposal.remove_proposal().removed();
                                if let Some(member) =
                                    group.members().find(|m| m.index == removed_index)
                                {
                                    let removed_id =
                                        SentinelIdentity::from_mls_credential(member.credential)?;
                                    removed_members.push(removed_id);
                                }
                            }

                            group.merge_staged_commit(&provider, staged_commit)?;

                            // Store vault messages after merge
                            if !added_members.is_empty() {
                                let decrypted_message = GroupMessage::new(
                                    MessageId::new(),
                                    sender.clone(),
                                    group_id.clone(),
                                    GroupMessageContent::UsersAdded(added_members),
                                    false,
                                    msg.published_at,
                                );
                                decrypted_messages.push(decrypted_message.clone());

                                self.vault
                                    .insert_j2j_message(&mut tx, &decrypted_message)
                                    .await?;
                            }

                            if !removed_members.is_empty() {
                                let decrypted_message = GroupMessage::new(
                                    MessageId::new(),
                                    sender.clone(),
                                    group_id.clone(),
                                    GroupMessageContent::UsersRemoved(removed_members.clone()),
                                    false,
                                    msg.published_at,
                                );
                                decrypted_messages.push(decrypted_message.clone());

                                self.vault
                                    .insert_j2j_message(&mut tx, &decrypted_message)
                                    .await?;
                            }
                        }
                        ProcessedMessageContent::OwnPendingCommit => {
                            // Own commit fanned back by the delivery service — merge the pending commit
                            group.merge_pending_commit(&provider)?;
                        }
                        ProcessedMessageContent::OwnPrivateMessage => {
                            // Own private message fanned back — content can't be decrypted, skip
                            tracing::debug!(
                                "Skipping own private message fanned back by delivery service"
                            );
                        }
                    }
                }
                _ => {
                    anyhow::bail!("Unexpected message type: expected Welcome or PrivateMessage");
                }
            }

            // After processing the message, update the max message id so that we don't attempt to process
            // it again, even if processing subsequent messages results in an error.
            self.vault
                .set_max_delivery_service_message_id(&mut tx, msg.message_id as u32)
                .await?;

            tx.commit().await?;
        }

        Ok(decrypted_messages)
    }

    /// Update every MLS group the client is a member of with a new LeafNode containing the new signature key,
    /// and send the resulting commit messages to the delivery service.
    /// https://book.openmls.tech/user_manual/updates.html
    async fn rotate_signature_key(
        &mut self,
        new_id_key_pair: SentinelIdKeyPair,
    ) -> anyhow::Result<()> {
        // Generate new credential and signer, but keep reference to old signer
        let (credential_with_key, signer) = generate_credential_and_signature_key_pair(
            MLS_CIPHERSUITE,
            &self.client_id,
            new_id_key_pair.clone(),
        );

        // Inform each group of the new leaf node information by updating the credential and signature key in the leaf node
        // TODO what if there is a partial failure here? Can we add an endpoint to send a batch of messages to the DS, so that
        // the update is a single atomic operation?
        // TODO all existing key packages are now invalid, so in this same request we should delete all existing key packages
        // for this client.
        // https://github.com/guardian/coverdrop-internal/issues/3898
        for group_id in &self.group_ids().await? {
            let mut tx = self.vault.pool.begin().await?;
            let provider = VaultProvider::new(&mut tx);

            let mut group = MlsGroup::load(provider.storage(), &group_id.to_open_mls_group_id())?
                .ok_or_else(|| anyhow::anyhow!("Group not found: {:?}", group_id))?;

            let new_signer_bundle = NewSignerBundle {
                signer: &signer,
                credential_with_key: credential_with_key.clone(),
            };

            let (mls_message_out, _welcome_option, _group_info) = group
                .self_update_with_new_signer(
                    &provider,
                    &self.signer,
                    new_signer_bundle,
                    LeafNodeParameters::default(),
                )
                .expect("Could not update own key package.")
                .into_contents();
            self.send_mls_message_to_group(&mut group, mls_message_out)
                .await
                .expect("Failed to send self-update commit message");

            // Merge the pending commit to advance this client's local group state to the new epoch
            group.merge_pending_commit(&provider)?;

            tx.commit().await?;
        }

        // now update self with new credentials after all groups have been updated
        self.client_id_key = new_id_key_pair;
        self.credential_with_key = credential_with_key;
        self.signer = signer;

        Ok(())
    }

    /// Check if the MLS group is active in the OpenMLS storage provider.
    /// Returns false if the group doesn't exist or if its state is Inactive
    /// (i.e. this client has been removed from the group).
    #[cfg(feature = "integration-tests")]
    pub async fn has_active_mls_group(&self, group_id: &GroupId) -> anyhow::Result<bool> {
        let mut conn = self.vault.pool.acquire().await?;
        let provider = VaultProvider::new(&mut conn);
        let group_exists = MlsGroup::load(provider.storage(), &group_id.to_open_mls_group_id())
            .ok()
            .flatten()
            .map(|group| group.is_active())
            .unwrap_or(false);

        Ok(group_exists)
    }

    fn get_group_members_from_group_id(
        &self,
        provider: &VaultProvider,
        group_id: &GroupId,
    ) -> anyhow::Result<Vec<SentinelIdentity>> {
        let group = MlsGroup::load(provider.storage(), &group_id.to_open_mls_group_id())?
            .ok_or_else(|| anyhow::anyhow!("Group not found"))?;

        let group_members = self.get_group_member_ids(&group, false)?;

        Ok(group_members)
    }

    pub async fn get_group_with_messages(&self, group_id: GroupId) -> anyhow::Result<Group> {
        let group_with_messages = self.vault.get_group_with_messages(&group_id).await?;

        let mut conn = self.vault.pool.acquire().await?;
        let provider = VaultProvider::new(&mut conn);

        let group = Group::new(
            group_with_messages.id,
            group_with_messages.display_name,
            group_with_messages.description,
            self.get_group_members_from_group_id(&provider, &group_id)?,
            group_with_messages.messages,
        );

        Ok(group)
    }

    /// Get a list of all groups, with group info, members and messages
    pub async fn get_groups_and_messages(&self) -> anyhow::Result<Vec<Group>> {
        let groups_with_messages = self.vault.get_groups_and_messages().await?;

        let mut conn = self.vault.pool.acquire().await?;
        let provider = VaultProvider::new(&mut conn);

        let mut groups = Vec::new();

        // TODO this is very inefficient because its a separate database query to load each OpenMLS group.
        // We should cache group membership information in memory and do this only once.
        // https://github.com/guardian/coverdrop-internal/issues/4055
        for group_with_messages in groups_with_messages {
            let group_members =
                self.get_group_members_from_group_id(&provider, &group_with_messages.id)?;

            groups.push(Group::new(
                group_with_messages.id,
                group_with_messages.display_name,
                group_with_messages.description,
                group_members,
                group_with_messages.messages,
            ));
        }

        Ok(groups)
    }

    pub async fn update_group_messages_read_status(
        &self,
        group_id: &GroupId,
        message_ids: Vec<MessageId>,
        read: bool,
    ) -> anyhow::Result<()> {
        self.vault
            .update_group_messages_read_status(group_id, &message_ids, read)
            .await
    }
}

/// Generate a credential and signature key pair for a client
fn generate_credential_and_signature_key_pair(
    ciphersuite: Ciphersuite,
    client_id: &SentinelIdentity,
    client_id_key: SentinelIdKeyPair,
) -> (CredentialWithKey, SignatureKeyPair) {
    let identity_bytes = client_id.as_bytes().to_vec();
    let credential = BasicCredential::new(identity_bytes);

    // TODO add a SentinelIdKeyPair::to_mls_signature_key_pair method
    let open_mls_signature_key = SignatureKeyPair::from_raw(
        ciphersuite.signature_algorithm(),
        client_id_key.secret_key.to_bytes().to_vec(),
        client_id_key.raw_public_key().to_bytes().to_vec(),
    );
    let credential_with_key = CredentialWithKey {
        credential: credential.into(),
        signature_key: open_mls_signature_key.public().into(),
    };

    (credential_with_key, open_mls_signature_key)
}

/// Authenticate a message received from the Delivery Service by verifying that the
/// credential and signing public key are present in the the trusted public key hierarchy.
/// https://book.openmls.tech/user_manual/credential_validation.html
fn authenticate_credential_and_key(
    sender_credential: Credential,
    sender_public_key: &SignaturePublicKey,
    public_keys: &OrganizationPublicKeyFamilyList,
    expected_sender: Option<&SentinelIdentity>,
) -> Result<(), anyhow::Error> {
    let credential_sentinel_id = SentinelIdentity::from_mls_credential(sender_credential)?;

    let sentinel_id = public_keys
        .find_sentinel_id_from_pk_bytes(sender_public_key.as_slice())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Failed to find matching sentinel ID public key for key package credential"
            )
        })?;

    // compare the sentinel ID to the key package credential identity
    if sentinel_id != &credential_sentinel_id {
        return Err(anyhow::anyhow!(
            "Sentinel ID does not match key package credential identity"
        ));
    }

    if let Some(expected_sender) = expected_sender {
        if sentinel_id != expected_sender {
            return Err(anyhow::anyhow!(
                "Sentinel ID does not match expected sender"
            ));
        }
    }

    Ok(())
}
