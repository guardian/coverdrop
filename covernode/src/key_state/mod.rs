mod inner_key_state;

use std::sync::Arc;

use chrono::{DateTime, Utc};
use common::{api::api_client::ApiClient, clap::Stage};
use covernode_database::Database;
use inner_key_state::{IdentityKeyPairCollection, MessagingKeyPairCollection};
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::{
    key_helpers::{get_and_verify_covernode_id_key_pairs, get_and_verify_covernode_msg_key_pairs},
    recipient_tag_lookup_table::RecipientTagKeyLookupTable,
};

pub use inner_key_state::InnerKeyState;

/// `KeyState` keeps track of all the local unpublished candidate identity and messaging keys as well as the published ones,
/// and has helper methods to use those keys for encryption and decryption.
///
/// This struct is cheaply clonable and contains a `RwLock` so that the key state can be shared across multiple threads.
#[derive(Clone)]
pub struct KeyState {
    inner: Arc<RwLock<InnerKeyState>>,
}

impl KeyState {
    pub async fn new(
        db: Database,
        api_client: &ApiClient,
        stage: &Stage,
        now: DateTime<Utc>,
    ) -> anyhow::Result<KeyState> {
        let anchor_org_pks = trust_anchors::get_trust_anchors(stage, now)?;

        let keys_and_profiles = api_client
            .get_public_keys()
            .await?
            .into_trusted(&anchor_org_pks, now);

        let keys = keys_and_profiles.keys;

        let covernode_provisioning_pks = keys
            .covernode_provisioning_pk_iter()
            .cloned()
            .collect::<Vec<_>>();

        if covernode_provisioning_pks.is_empty() {
            anyhow::bail!("No valid provisioning keys in API, cannot start CoverNode");
        }

        let candidate_covernode_id_key_pair = db
            .select_candidate_id_key_pair()
            .await?
            .map(|k| k.key_pair.to_trusted());

        let covernode_id_key_pairs =
            get_and_verify_covernode_id_key_pairs(&db, &covernode_provisioning_pks, now).await?;

        let candidate_covernode_msg_key_pair =
            db.select_candidate_msg_key_pair().await?.and_then(|k| {
                k.key_pair
                    .to_trusted_from_candidate_parents(covernode_id_key_pairs.iter(), now)
                    .ok()
            });
        let covernode_msg_key_pairs =
            get_and_verify_covernode_msg_key_pairs(&db, &covernode_id_key_pairs, now).await?;

        let covernode_id_key_pairs =
            IdentityKeyPairCollection::new(candidate_covernode_id_key_pair, covernode_id_key_pairs);

        let covernode_msg_key_pairs = MessagingKeyPairCollection::new(
            candidate_covernode_msg_key_pair,
            covernode_msg_key_pairs,
        );

        let covernode_id_key_pairs = covernode_id_key_pairs;
        let covernode_msg_key_pairs = covernode_msg_key_pairs;

        let tag_lookup_table =
            RecipientTagKeyLookupTable::from_api(api_client, &anchor_org_pks, now).await?;

        let mut inner = InnerKeyState::new(
            api_client.clone(),
            db,
            anchor_org_pks,
            covernode_id_key_pairs,
            covernode_msg_key_pairs,
            tag_lookup_table,
        );

        inner.process_setup_bundle(&keys, now).await?;

        Ok(KeyState {
            inner: Arc::new(RwLock::new(inner)),
        })
    }

    pub async fn read(&self) -> RwLockReadGuard<'_, InnerKeyState> {
        self.inner.read().await
    }

    pub async fn write(&self) -> RwLockWriteGuard<'_, InnerKeyState> {
        self.inner.write().await
    }
}
