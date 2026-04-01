use anyhow::bail;
use common::api::models::journalist_id::JournalistIdentity;
use common::crypto::keys::signing::traits::PublicSigningKey;
use common::protocol::keys::{JournalistIdKeyPair, OrganizationPublicKeyFamilyList};
use common::time;
use delivery_service_lib::client::DeliveryServiceClient;
use delivery_service_lib::forms::{
    AddMembersForm, ConsumeKeyPackageForm, GetClientsForm, PublishKeyPackagesForm,
    ReceiveMessagesForm, RegisterClientForm, SendMessageForm,
};
use delivery_service_lib::models::KeyPackageWithClientId;
use delivery_service_lib::tls_serialized::TlsSerialized;
use delivery_service_lib::{MLS_CIPHERSUITE, PROTOCOL_VERSION};
use openmls::prelude::*;
use openmls_basic_credential::SignatureKeyPair;
use openmls_rust_crypto::OpenMlsRustCrypto;
use openmls_traits::OpenMlsProvider;
use reqwest::Url;
use std::collections::HashMap;

/// Test client for MLS group messaging operations
pub struct TestClient {
    pub client_id: JournalistIdentity,
    pub client_id_key: JournalistIdKeyPair,
    pub credential_with_key: CredentialWithKey,
    pub signer: SignatureKeyPair,
    pub crypto: OpenMlsRustCrypto,
    pub ciphersuite: Ciphersuite,
    pub groups: HashMap<Vec<u8>, MlsGroup>,
    pub delivery_service_client: DeliveryServiceClient,
    pub last_message_id: u32,
}

/// Client for testing group messaging operations with the delivery service.
/// TODO move this functionality to GroupMessagingService in journalist-services/
/// https://github.com/guardian/coverdrop-internal/issues/3889
impl TestClient {
    /// Create a new test client with a given identity
    pub fn new(
        client_id: &JournalistIdentity,
        client_id_key: JournalistIdKeyPair,
        delivery_service_url: Url,
    ) -> Self {
        let (credential_with_key, signer) = generate_credential_and_signature_key_pair(
            MLS_CIPHERSUITE,
            client_id,
            client_id_key.clone(),
        );

        Self {
            client_id: client_id.clone(),
            client_id_key,
            credential_with_key,
            signer,
            crypto: OpenMlsRustCrypto::default(),
            ciphersuite: MLS_CIPHERSUITE,
            groups: HashMap::new(),
            delivery_service_client: DeliveryServiceClient::new(delivery_service_url),
            last_message_id: 0,
        }
    }

    /// Generate key packages for this client and return them along with their hashes.
    /// The hash is used by the delivery service as a unique identifier for each key package.
    fn generate_key_packages(&self, count: usize) -> anyhow::Result<Vec<KeyPackageIn>> {
        (0..count)
            .map(|_| {
                let key_package_bundle = KeyPackage::builder()
                    .key_package_extensions(Extensions::empty())
                    .build(
                        self.ciphersuite,
                        &self.crypto,
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
        let key_packages = self.generate_key_packages(num_key_packages)?;
        let form = RegisterClientForm::new(key_packages, &self.client_id_key, time::now())?;
        self.delivery_service_client.register_client(form).await
    }

    /// Publish additional key packages to the delivery service
    pub async fn publish_key_packages(&self, num_key_packages: usize) -> anyhow::Result<()> {
        let key_packages = self.generate_key_packages(num_key_packages)?;
        let form = PublishKeyPackagesForm::new(key_packages, &self.client_id_key, time::now())?;
        self.delivery_service_client
            .publish_key_packages(form)
            .await
    }

    /// Get the list of registered clients
    pub async fn get_clients(&self) -> anyhow::Result<Vec<JournalistIdentity>> {
        let form = GetClientsForm::new(&self.client_id_key, time::now())?;

        self.delivery_service_client.get_clients(form).await
    }

    /// Get a key package for another client from the delivery service and verify its authenticity
    pub async fn get_key_package(
        &self,
        target_client_id: &JournalistIdentity,
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
        let validated_key_package = key_package.validate(self.crypto.crypto(), PROTOCOL_VERSION)?;

        Ok(KeyPackageWithClientId {
            client_id: target_client_id.clone(),
            key_package: validated_key_package,
        })
    }

    /// Create a new MLS group
    pub fn create_group(&mut self, group_id: Vec<u8>) -> anyhow::Result<()> {
        let group_config = &MlsGroupCreateConfig::builder()
            .use_ratchet_tree_extension(true)
            .build();
        let group = MlsGroup::new_with_group_id(
            &self.crypto,
            &self.signer,
            group_config,
            GroupId::from_slice(&group_id),
            self.credential_with_key.clone(),
        )?;

        self.groups.insert(group_id, group);
        Ok(())
    }

    /// Add members to a group and send Welcome messages
    pub async fn add_members(
        &mut self,
        group_id: &[u8],
        key_packages_with_client_ids: Vec<KeyPackageWithClientId>,
    ) -> anyhow::Result<()> {
        let group = self
            .groups
            .get_mut(group_id)
            .ok_or_else(|| anyhow::anyhow!("Group not found"))?;

        let key_packages = key_packages_with_client_ids
            .iter()
            .map(|kp| kp.key_package.clone())
            .collect::<Vec<_>>();

        let new_members = key_packages_with_client_ids
            .iter()
            .map(|kp| kp.client_id.clone())
            .collect::<Vec<_>>();

        let (mls_message_out, welcome, _) =
            group.add_members(&self.crypto, &self.signer, &key_packages)?;

        // Get the list of existing members (everyone except the new members being added)
        let existing_members: Vec<JournalistIdentity> =
            Self::get_group_members(group, &self.client_id)?;

        // Serialize the welcome message
        let welcome_message = TlsSerialized::serialize(&welcome)?;

        // Serialize the commit message for existing members
        let commit_message = TlsSerialized::serialize(&mls_message_out)?;

        let form = AddMembersForm::new(
            welcome_message,
            commit_message,
            existing_members,
            new_members,
            &self.client_id_key,
            time::now(),
        )?;

        self.delivery_service_client.add_members(form).await?;

        group.merge_pending_commit(&self.crypto)?;

        Ok(())
    }

    /// returns the list of JournalistIdentities of the members of the group
    /// excluding the client itself.
    fn get_group_members(
        group: &MlsGroup,
        client_id: &JournalistIdentity,
    ) -> anyhow::Result<Vec<JournalistIdentity>> {
        let members = group
            .members()
            .filter_map(|member| {
                // Parse the identity from the credential
                if let Ok(basic_cred) = BasicCredential::try_from(member.credential.clone()) {
                    // TODO - create JournalistIdentity constructor from BasicCredential
                    let journalist_id_str =
                        String::from_utf8_lossy(basic_cred.identity()).to_string();
                    JournalistIdentity::new(&journalist_id_str).ok()
                } else {
                    None
                }
            })
            // Filter out the member who is adding (self)
            .filter(|id| id != client_id)
            .collect();

        Ok(members)
    }

    /// Send an application message to the group
    pub async fn send_message(&mut self, group_id: &[u8], message: &[u8]) -> anyhow::Result<()> {
        let group = self
            .groups
            .get_mut(group_id)
            .ok_or_else(|| anyhow::anyhow!("Group not found"))?;

        let mls_message_out = group.create_message(&self.crypto, &self.signer, message)?;
        Self::send_mls_message_out(
            group,
            mls_message_out,
            &self.client_id,
            &self.client_id_key,
            &self.delivery_service_client,
        )
        .await
    }

    async fn send_mls_message_out(
        group: &mut MlsGroup,
        mls_message_out: MlsMessageOut,
        client_id: &JournalistIdentity,
        client_id_key: &JournalistIdKeyPair,
        delivery_service_client: &DeliveryServiceClient,
    ) -> anyhow::Result<()> {
        let recipients = Self::get_group_members(group, client_id)?;
        let message = TlsSerialized::serialize(&mls_message_out)?;

        let form = SendMessageForm::new(message, recipients, client_id_key, time::now())?;

        delivery_service_client.send_message(form).await
    }

    /// Receive and process messages from the delivery service
    pub async fn receive_messages(
        &mut self,
        public_keys: &OrganizationPublicKeyFamilyList,
    ) -> anyhow::Result<Vec<Vec<u8>>> {
        let form =
            ReceiveMessagesForm::new(self.last_message_id, &self.client_id_key, time::now())?;

        let messages = self.delivery_service_client.receive_messages(form).await?;

        // TODO once a message is processed, it can't be decrypted again, and attempts to do so return
        // a SecretReuseError. Handle this as part of https://github.com/guardian/coverdrop-internal/issues/3918
        // by having the DS store the epoch of the key package / message, then not attempting to authenticate and
        // process key packages or messages using a stale key hierarchy.

        let mut decrypted_messages = Vec::new();

        for msg in messages {
            // Deserialize the MLS message and extract its body
            let msg_bytes = &msg.content;
            let mls_message_in = msg_bytes.deserialize::<MlsMessageIn>()?;
            let mls_message_body = mls_message_in.extract();

            // TODO break into helper functions for processing different message types
            match mls_message_body {
                MlsMessageBodyIn::Welcome(welcome) => {
                    // We're skipping the ProcessedWelcome stage for the moment since we don't need
                    // to retrieve information from the `Welcome` about the ratchet tree and PSKs
                    let group_join_config = MlsGroupJoinConfig::default();
                    let staged_welcome = StagedWelcome::new_from_welcome(
                        &self.crypto,
                        &group_join_config,
                        welcome,
                        None,
                    )?;

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

                    let group = staged_welcome.into_group(&self.crypto)?;

                    let group_id = group.group_id().as_slice().to_vec();
                    self.groups.insert(group_id, group);
                }
                MlsMessageBodyIn::PrivateMessage(private_msg) => {
                    // All handshake and application messages use PrivateMessage (encrypted)
                    let protocol_message = ProtocolMessage::from(private_msg);

                    let group_id = protocol_message.group_id();
                    let group = match self.groups.get_mut(group_id.as_slice()) {
                        Some(g) => g,
                        None => bail!("Received message for unknown group: {:?}", group_id),
                    };

                    // AUTH processing performs all syntactic and semantic validation checks and verifies the message’s signature
                    // https://book.openmls.tech/user_manual/processing.html#processing-messages-in-groups
                    let processed = group.process_message(&self.crypto, protocol_message)?;

                    match processed.into_content() {
                        ProcessedMessageContent::ApplicationMessage(app_msg) => {
                            let message_bytes = app_msg.into_bytes();
                            decrypted_messages.push(message_bytes);
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
                            for queued_add_proposal in staged_commit.add_proposals() {
                                let new_member_key_package =
                                    queued_add_proposal.add_proposal().key_package();
                                let leaf_node = new_member_key_package.leaf_node();
                                let credential = leaf_node.credential().clone();
                                let signature_key = leaf_node.signature_key();

                                authenticate_credential_and_key(
                                    credential,
                                    signature_key,
                                    public_keys,
                                    None,
                                )?;
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

                            group.merge_staged_commit(&self.crypto, staged_commit)?;
                        }
                    }
                }
                _ => {
                    anyhow::bail!("Unexpected message type: expected Welcome or PrivateMessage");
                }
            }

            // after processing the message, update the last_message_id to the id of the received message
            // so that we don't attempt to process it again.
            self.last_message_id = self.last_message_id.max(msg.message_id as u32);
        }

        Ok(decrypted_messages)
    }

    /// Update every MLS group the client is a member of with a new LeafNode containing the new signature key,
    /// and send the resulting commit messages to the delivery service.
    /// https://book.openmls.tech/user_manual/updates.html
    pub async fn rotate_signature_key(
        &mut self,
        new_id_key_pair: JournalistIdKeyPair,
    ) -> anyhow::Result<()> {
        // Generate new credential and signer, but keep reference to old signer
        let (credential_with_key, signer) = generate_credential_and_signature_key_pair(
            self.ciphersuite,
            &self.client_id,
            new_id_key_pair.clone(),
        );

        // Inform each group of the new leaf node information by updating the credential and signature key in the leaf node
        // TODO what if there is a partial failure here? Can we add an endpoint to send a batch of messages to the DS, so that
        // the update is a single atomic operation? https://github.com/guardian/coverdrop-internal/issues/3898
        for group in self.groups.values_mut() {
            let new_signer_bundle = NewSignerBundle {
                signer: &signer,
                credential_with_key: credential_with_key.clone(),
            };

            let (mls_message_out, _welcome_option, _group_info) = group
                .self_update_with_new_signer(
                    &self.crypto,
                    &self.signer,
                    new_signer_bundle,
                    LeafNodeParameters::default(),
                )
                .expect("Could not update own key package.")
                .into_contents();
            Self::send_mls_message_out(
                group,
                mls_message_out,
                &self.client_id,
                &self.client_id_key,
                &self.delivery_service_client,
            )
            .await
            .expect("Failed to send self-update commit message");

            // Merge the pending commit to advance this client's local group state to the new epoch
            group.merge_pending_commit(&self.crypto)?;
        }

        // now update self with new credentials after all groups have been updated
        self.client_id_key = new_id_key_pair;
        self.credential_with_key = credential_with_key;
        self.signer = signer;

        Ok(())
    }
}

/// Generate a credential and signature key pair for a client
pub fn generate_credential_and_signature_key_pair(
    ciphersuite: Ciphersuite,
    client_id: &JournalistIdentity,
    client_id_key: JournalistIdKeyPair,
) -> (CredentialWithKey, SignatureKeyPair) {
    let identity_bytes = client_id.as_bytes().to_vec();
    let credential = BasicCredential::new(identity_bytes);

    // TODO should be using sentinel key
    // TODO add a SentinelIdKeyPair::to_mls_signature_key_pair method
    // https://github.com/guardian/coverdrop-internal/issues/3888
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
    expected_sender: Option<&JournalistIdentity>,
) -> Result<(), anyhow::Error> {
    let sender_basic_credential = BasicCredential::try_from(sender_credential).map_err(|_| {
        anyhow::anyhow!("Failed to parse key package credential as BasicCredential")
    })?;

    let journalist_id = public_keys
        .find_journalist_id_from_pk_bytes(sender_public_key.as_slice())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Failed to find matching journalist ID public key for key package credential"
            )
        })?;

    // compare the journalist ID to the key package credential identity
    if journalist_id.as_bytes().to_vec() != sender_basic_credential.identity() {
        return Err(anyhow::anyhow!(
            "Journalist ID does not match key package credential identity"
        ));
    }

    if let Some(expected_sender) = expected_sender {
        if journalist_id != expected_sender {
            return Err(anyhow::anyhow!(
                "Journalist ID does not match expected sender"
            ));
        }
    }

    Ok(())
}
