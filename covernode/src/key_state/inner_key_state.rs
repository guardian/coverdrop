use chrono::{DateTime, Utc};
use common::{
    api::{api_client::ApiClient, forms::PostCoverNodeIdPublicKeyForm},
    crypto::keys::signed::SignedKey,
    epoch::Epoch,
    protocol::{
        keys::{
            AnchorOrganizationPublicKey, CoverDropPublicKeyHierarchy, CoverNodeIdKeyPair,
            CoverNodeIdKeyPairWithEpoch, CoverNodeMessagingKeyPair,
            CoverNodeMessagingKeyPairWithEpoch, JournalistMessagingPublicKey,
            UnregisteredCoverNodeIdKeyPair,
        },
        recipient_tag::RecipientTag,
    },
};
use covernode_database::{
    Database, UntrustedCandidateCoverNodeIdKeyPairWithCreatedAt,
    UntrustedCandidateCoverNodeMessagingKeyPairWithCreatedAt,
    UntrustedCoverNodeIdKeyPairWithCreatedAt,
};

use crate::recipient_tag_lookup_table::RecipientTagKeyLookupTable;

// Separates out the candidate keys, which don't have an epoch assigned from
// the published keys, which do have an epoch
pub struct IdentityKeyPairCollection {
    candidate: Option<UnregisteredCoverNodeIdKeyPair>,
    published: Vec<CoverNodeIdKeyPairWithEpoch>,
}

impl IdentityKeyPairCollection {
    pub fn new(
        candidate: Option<UnregisteredCoverNodeIdKeyPair>,
        published: Vec<CoverNodeIdKeyPairWithEpoch>,
    ) -> Self {
        Self {
            candidate,
            published,
        }
    }
}

pub struct MessagingKeyPairCollection {
    candidate: Option<CoverNodeMessagingKeyPair>,
    published: Vec<CoverNodeMessagingKeyPairWithEpoch>,
}

impl MessagingKeyPairCollection {
    pub fn new(
        candidate: Option<CoverNodeMessagingKeyPair>,
        published: Vec<CoverNodeMessagingKeyPairWithEpoch>,
    ) -> Self {
        Self {
            candidate,
            published,
        }
    }
}

// See `KeyState` docs for more info.
pub struct InnerKeyState {
    api_client: ApiClient,
    db: Database,

    anchor_org_pks: Vec<AnchorOrganizationPublicKey>,
    covernode_id_key_pairs: IdentityKeyPairCollection,
    covernode_msg_key_pairs: MessagingKeyPairCollection,

    tag_lookup_table: RecipientTagKeyLookupTable,
}

impl InnerKeyState {
    pub fn new(
        api_client: ApiClient,
        db: Database,
        anchor_org_pks: Vec<AnchorOrganizationPublicKey>,
        covernode_id_key_pairs: IdentityKeyPairCollection,
        mut covernode_msg_key_pairs: MessagingKeyPairCollection,
        tag_lookup_table: RecipientTagKeyLookupTable,
    ) -> Self {
        covernode_msg_key_pairs
            .published
            .sort_by_key(|key_pair_with_epoch| key_pair_with_epoch.not_valid_after());

        covernode_msg_key_pairs.published.reverse();

        Self {
            api_client,
            db,
            anchor_org_pks,
            covernode_id_key_pairs,
            covernode_msg_key_pairs,
            tag_lookup_table,
        }
    }

    pub async fn refresh_tag_lookup_table(&mut self, now: DateTime<Utc>) -> anyhow::Result<()> {
        let tag_lookup_table =
            RecipientTagKeyLookupTable::from_api(&self.api_client, &self.anchor_org_pks, now)
                .await?;

        self.tag_lookup_table = tag_lookup_table;

        tracing::debug!("Updated CoverNode state cache keys");

        Ok(())
    }

    //
    // Encryption and decryption
    //
    pub async fn latest_journalist_msg_pk_from_recipient_tag(
        &self,
        tag: &RecipientTag,
    ) -> Option<JournalistMessagingPublicKey> {
        self.tag_lookup_table.get(tag).cloned()
    }

    pub fn anchor_org_pks(&self) -> &[AnchorOrganizationPublicKey] {
        &self.anchor_org_pks
    }

    pub fn covernode_msg_key_pairs_for_decryption_with_rank(
        &self,
        now: DateTime<Utc>,
    ) -> impl Iterator<Item = (usize, &CoverNodeMessagingKeyPair)> {
        let msg_key_pairs = &self.covernode_msg_key_pairs;

        // We want the ranks to be consistent and meaningful across different sets of keys
        // so that we can see if clients are using the latest published key (expected),
        // an older key (bad) or a candidate key (really bad, the CoverNode doesn't realise a key is available)
        //
        // I will define the candidate key as always being 0 and then published keys being 1, 2, 3. Going from latest
        // to oldest. It's important that the keys are decending order since we may have a small period where there is one more
        // or one fewer keys than normal since an expiry has happened before a publication of a new key.
        // If we have no candidate key we manually bump the enumeration value by 1 so that there is no zeroth key.
        let bump_enumeration = if msg_key_pairs.candidate.is_some() {
            0
        } else {
            1
        };

        let candidate_key_pair_iter = msg_key_pairs.candidate.iter();
        let published_key_pair_iter = msg_key_pairs
            .published
            .iter()
            .map(|key_pair_with_epoch| &key_pair_with_epoch.key_pair);

        candidate_key_pair_iter
            .chain(published_key_pair_iter)
            .enumerate()
            .map(move |(rank, key_pair)| (rank + bump_enumeration, key_pair))
            .filter(move |(_, key_pair)| !key_pair.is_not_valid_after(now))
    }

    pub fn candidate_covernode_id_key_pair(&self) -> &Option<UnregisteredCoverNodeIdKeyPair> {
        &self.covernode_id_key_pairs.candidate
    }

    pub fn published_covernode_id_key_pairs(&self) -> &[CoverNodeIdKeyPairWithEpoch] {
        &self.covernode_id_key_pairs.published
    }

    pub fn candidate_covernode_msg_key_pair(&self) -> &Option<CoverNodeMessagingKeyPair> {
        &self.covernode_msg_key_pairs.candidate
    }

    pub fn published_covernode_msg_key_pairs(&self) -> &[CoverNodeMessagingKeyPairWithEpoch] {
        &self.covernode_msg_key_pairs.published
    }

    //
    // Database wrappers
    // These are required because we need to maintain in-memory state as
    // we manipulate the database
    //

    pub async fn insert_candidate_id_key_pair(
        &mut self,
        key_pair: UnregisteredCoverNodeIdKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        if self.covernode_id_key_pairs.candidate.is_some() {
            anyhow::bail!("Candidate id key pair already set");
        }
        self.db
            .insert_candidate_id_key_pair(&key_pair, now)
            .await
            .inspect_err(|e| {
                tracing::error!("Could not insert candidate id pair into database {}", e)
            })?;
        self.covernode_id_key_pairs.candidate = Some(key_pair);
        Ok(())
    }

    pub async fn insert_candidate_msg_key_pair(
        &mut self,
        key_pair: CoverNodeMessagingKeyPair,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        if self.covernode_msg_key_pairs.candidate.is_some() {
            anyhow::bail!("Candidate msg key pair already set");
        }
        self.db
            .insert_candidate_msg_key_pair(&key_pair, now)
            .await
            .inspect_err(|e| {
                tracing::error!("Could not insert candidate msg pair into database {}", e)
            })?;
        self.covernode_msg_key_pairs.candidate = Some(key_pair);
        Ok(())
    }

    pub async fn add_epoch_to_covernode_id_key_pair(
        &mut self,
        id_key_pair: CoverNodeIdKeyPair,
        epoch: Epoch,
        key_created_at: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        self.db
            .update_candidate_id_key_pair_add_epoch(&id_key_pair, epoch)
            .await
            .map_err(|e| {
                tracing::error!("could not update id key pair in database {}", e);
                e
            })?;

        self.covernode_id_key_pairs
            .published
            .push(CoverNodeIdKeyPairWithEpoch::new(
                id_key_pair,
                epoch,
                key_created_at,
            ));

        self.covernode_id_key_pairs.candidate = None;

        Ok(())
    }

    // This is used by the setup bundle. It inserts a key pair
    // and its epoch directly without going via the identity api
    pub async fn insert_covernode_id_key_pair(
        &mut self,
        id_key_pair: CoverNodeIdKeyPair,
        epoch: Epoch,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        self.db
            .insert_id_key_pair_with_epoch(&id_key_pair, epoch, now)
            .await
            .inspect_err(|e| {
                tracing::error!("could not insert id key pair in database {}", e);
            })?;

        self.covernode_id_key_pairs
            .published
            .push(CoverNodeIdKeyPairWithEpoch::new(id_key_pair, epoch, now));

        Ok(())
    }

    pub async fn add_epoch_to_covernode_msg_key_pair(
        &mut self,
        msg_key_pair: CoverNodeMessagingKeyPair,
        epoch: Epoch,
        key_created_at: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        self.db
            .update_candidate_msg_key_pair_add_epoch(&msg_key_pair.to_untrusted(), epoch)
            .await
            .inspect_err(|e| {
                tracing::error!("could not update msg key pair in database {}", e);
            })?;

        self.covernode_msg_key_pairs
            .published
            .push(CoverNodeMessagingKeyPairWithEpoch::new(
                msg_key_pair,
                epoch,
                key_created_at,
            ));

        self.covernode_msg_key_pairs
            .published
            .sort_by_key(|key_pair_with_epoch| key_pair_with_epoch.not_valid_after());

        // Verse the sort order of the messaging keys so that the latest one is used first
        // This is because most messages coming in will be used with this most recent key
        // and the decryption function will early exit if the right key is found.
        self.covernode_msg_key_pairs.published.reverse();

        self.covernode_msg_key_pairs.candidate = None;

        Ok(())
    }

    pub async fn get_setup_bundle(
        &self,
    ) -> anyhow::Result<
        Option<(
            PostCoverNodeIdPublicKeyForm,
            UntrustedCoverNodeIdKeyPairWithCreatedAt,
        )>,
    > {
        self.db.select_setup_bundle().await
    }
    pub async fn process_setup_bundle(
        &mut self,
        keys: &CoverDropPublicKeyHierarchy,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let setup_bundle = self.db.select_setup_bundle().await?;

        // An existing setup bundle key is a key that's already been generated and published
        // but covernode does not have it in its database.
        // covernode needs to re-publish this key to api in order to retrieve the epoch value
        // and insert the key in its database
        if let Some((form, untrusted_key_pair_with_created_at)) = setup_bundle {
            tracing::info!("Found a setup bundle!");

            let epoch = self.api_client.post_covernode_id_pk_form(&form).await?;
            tracing::info!("Successfully posted setup bundle CoverNode id public key form");

            let untrusted_key_pair = untrusted_key_pair_with_created_at.key_pair;

            let provisioning_key_pairs = keys.covernode_provisioning_pk_iter();
            let trusted_signed_id_key_pair = untrusted_key_pair
                .to_trusted_from_candidate_parents(provisioning_key_pairs, now)?;

            self.insert_covernode_id_key_pair(trusted_signed_id_key_pair, epoch, now)
                .await?;

            self.db.delete_setup_bundle().await?;

            tracing::info!("successfully processed setup bundle");
        } else {
            tracing::debug!("No setup bundle key found");

            let covernode_id_key_pairs = self.published_covernode_id_key_pairs();

            if covernode_id_key_pairs.is_empty() {
                let msg = "No setup bundle and no covernode id key pairs found in database - covernode cannot run";
                tracing::error!(msg);
                std::panic!("{}", msg);
            }
        }

        Ok(())
    }

    pub async fn get_candidate_id_key_pair(
        &self,
    ) -> anyhow::Result<Option<UntrustedCandidateCoverNodeIdKeyPairWithCreatedAt>> {
        self.db.select_candidate_id_key_pair().await
    }

    pub async fn get_candidate_msg_key_pair(
        &self,
    ) -> anyhow::Result<Option<UntrustedCandidateCoverNodeMessagingKeyPairWithCreatedAt>> {
        self.db.select_candidate_msg_key_pair().await
    }

    /// Returns the maxmium epoch of the currently cached CoverNode identity and messaging keys
    /// can return `None` if there are no keys of either kind.
    pub fn max_epoch(&self) -> Option<Epoch> {
        let covernode_msg_key_pairs_max_epoch = self
            .covernode_msg_key_pairs
            .published
            .iter()
            .max_by_key(|key_pair_with_epoch| key_pair_with_epoch.epoch)
            .map(|key_pair_with_epoch| key_pair_with_epoch.epoch);

        let covernode_id_key_pairs_max_epoch = self
            .covernode_id_key_pairs
            .published
            .iter()
            .max_by_key(|key_pair_with_epoch| key_pair_with_epoch.epoch)
            .map(|key_pair_with_epoch| key_pair_with_epoch.epoch);

        std::cmp::max(
            covernode_id_key_pairs_max_epoch,
            covernode_msg_key_pairs_max_epoch,
        )
    }

    pub async fn delete_expired_id_key_pairs(&mut self, now: DateTime<Utc>) -> anyhow::Result<()> {
        self.db.delete_expired_id_key_pairs(now).await?;

        // Should remove silly state management stuff
        // https://github.com/guardian/coverdrop-internal/issues/3045
        let to_remove = self
            .covernode_id_key_pairs
            .published
            .iter()
            .enumerate()
            .rev()
            .filter_map(|(i, k)| {
                if k.is_not_valid_after(now) {
                    Some(i)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        for i in to_remove {
            self.covernode_id_key_pairs.published.remove(i);
        }

        Ok(())
    }

    pub async fn delete_expired_msg_key_pairs(&mut self, now: DateTime<Utc>) -> anyhow::Result<()> {
        self.db.delete_expired_msg_key_pairs(now).await?;

        // Should remove silly state management stuff
        // https://github.com/guardian/coverdrop-internal/issues/3045
        let to_remove = self
            .covernode_msg_key_pairs
            .published
            .iter()
            .enumerate()
            .rev()
            .filter_map(|(i, k)| {
                if k.is_not_valid_after(now) {
                    Some(i)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        for i in to_remove {
            self.covernode_msg_key_pairs.published.remove(i);
        }
        Ok(())
    }
}
