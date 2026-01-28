//! An internal utility structs for combining keys with a numeric ID
//! from the database.
//!
//! In our database each keys have a reference to the keys that created them.
//! This is helpful when you want to join across a key hierarchy
//!
//! These data structures should never leave the `journalist-vault`
//! crate since the ID is meaningless outside of the relational model.

use chrono::{DateTime, Utc};
use common::{
    api::forms::{PostJournalistForm, PostJournalistIdPublicKeyForm},
    epoch::Epoch,
    protocol::keys::{
        AnchorOrganizationPublicKey, JournalistIdKeyPair, JournalistMessagingKeyPair,
        JournalistProvisioningPublicKey, UnregisteredJournalistIdKeyPair,
        UntrustedAnchorOrganizationPublicKey, UntrustedJournalistIdKeyPair,
        UntrustedJournalistMessagingKeyPair, UntrustedJournalistProvisioningPublicKey,
        UntrustedUnregisteredJournalistIdKeyPair,
    },
};
use serde::{ser::SerializeStruct as _, Serialize};

pub struct PublicKeyRow<PK> {
    pub id: i64,
    pub pk: PK,
}

impl<PK> PublicKeyRow<PK> {
    pub(crate) fn new(id: i64, pk: PK) -> Self {
        Self { id, pk }
    }

    pub fn into_public_key(self) -> PK {
        self.pk
    }
}

impl<PK> Serialize for PublicKeyRow<PK>
where
    PK: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("PublicKeyRow", 2)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("pk", &self.pk)?;
        state.end()
    }
}

pub(crate) type OrganizationPublicKeyRow = PublicKeyRow<AnchorOrganizationPublicKey>;
pub(crate) type JournalistProvisioningPublicKeyRow = PublicKeyRow<JournalistProvisioningPublicKey>;

// Unverified versions
pub type UntrustedOrganizationPublicKeyRow = PublicKeyRow<UntrustedAnchorOrganizationPublicKey>;
pub type UntrustedJournalistProvisioningPublicKeyRow =
    PublicKeyRow<UntrustedJournalistProvisioningPublicKey>;

//
// Key pair rows
//

pub struct CandidateKeyPairRow<T> {
    pub id: i64,
    pub added_at: DateTime<Utc>,
    pub key_pair: T,
}

impl<T> CandidateKeyPairRow<T> {
    pub(crate) fn new(id: i64, added_at: DateTime<Utc>, key_pair: T) -> Self {
        Self {
            id,
            added_at,
            key_pair,
        }
    }
}

impl<T> Serialize for CandidateKeyPairRow<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("CandidateKeyPairRow", 2)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("added_at", &self.added_at)?;
        state.serialize_field("key_pair", &self.key_pair)?;
        state.end()
    }
}

// Published rows - ones that we know are in the API

pub struct PublishedKeyPairRow<T> {
    pub id: i64,
    pub key_pair: T,
    pub epoch: Epoch,
}

impl<T> Serialize for PublishedKeyPairRow<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("PublishedKeyPairRow", 3)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("key_pair", &self.key_pair)?;
        state.serialize_field("epoch", &self.epoch)?;
        state.end()
    }
}

impl<T> PublishedKeyPairRow<T> {
    pub(crate) fn new(id: i64, key_pair: T, epoch: Epoch) -> Self {
        Self {
            id,
            key_pair,
            epoch,
        }
    }
}

pub(crate) type CandidateJournalistIdKeyPairRow =
    CandidateKeyPairRow<UnregisteredJournalistIdKeyPair>;
pub(crate) type CandidateJournalistMessagingKeyPairRow =
    CandidateKeyPairRow<JournalistMessagingKeyPair>;

pub(crate) type PublishedJournalistIdKeyPairRow = PublishedKeyPairRow<JournalistIdKeyPair>;
pub(crate) type PublishedJournalistMessagingKeyPairRow =
    PublishedKeyPairRow<JournalistMessagingKeyPair>;

// Unverified
pub type UntrustedCandidateJournalistIdKeyPairRow =
    CandidateKeyPairRow<UntrustedUnregisteredJournalistIdKeyPair>;
pub type UntrustedCandidateJournalistMessagingKeyPairRow =
    CandidateKeyPairRow<UntrustedJournalistMessagingKeyPair>;

pub type UntrustedPublishedJournalistIdKeyPairRow =
    PublishedKeyPairRow<UntrustedJournalistIdKeyPair>;
pub type UntrustedPublishedJournalistMessagingKeyPairRow =
    PublishedKeyPairRow<UntrustedJournalistMessagingKeyPair>;

pub(crate) struct SeedInfoRow {
    pub provisioning_pk_id: i64,
    pub pk_upload_form: PostJournalistIdPublicKeyForm,
    pub key_pair: JournalistIdKeyPair,
    pub register_journalist_form: Option<PostJournalistForm>,
}

impl SeedInfoRow {
    pub(crate) fn new(
        provisioning_pk_id: i64,
        pk_upload_form: PostJournalistIdPublicKeyForm,
        key_pair: JournalistIdKeyPair,
        register_journalist_form: Option<PostJournalistForm>,
    ) -> Self {
        Self {
            provisioning_pk_id,
            pk_upload_form,
            key_pair,
            register_journalist_form,
        }
    }
}

#[derive(Serialize)]
pub struct AllVaultKeys {
    pub org_pks: Vec<UntrustedAnchorOrganizationPublicKey>,
    pub journalist_provisioning_pks: Vec<UntrustedJournalistProvisioningPublicKeyRow>,

    pub candidate_id_key_pair: Option<UntrustedCandidateJournalistIdKeyPairRow>,
    pub candidate_msg_key_pair: Option<UntrustedCandidateJournalistMessagingKeyPairRow>,

    pub published_id_key_pairs: Vec<UntrustedPublishedJournalistIdKeyPairRow>,
    pub published_msg_key_pairs: Vec<UntrustedPublishedJournalistMessagingKeyPairRow>,
}
