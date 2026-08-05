use chrono::{DateTime, Utc};
use common::{
    crypto::keys::{public_key::PublicKey, signing::UnsignedSigningKeyPair},
    epoch::Epoch,
    identity_api::models::{
        UntrustedJournalistIdPublicKeyWithEpoch, UntrustedSentinelIdPublicKeyWithEpoch,
    },
    protocol::keys::{
        AnchorOrganizationPublicKeys, JournalistIdKeyPair, JournalistIdPublicKey,
        JournalistProvisioningPublicKey, LatestKey, SentinelIdKeyPair, SentinelIdPublicKey,
        UnregisteredJournalistIdKeyPair, UnregisteredSentinelIdKeyPair,
    },
};
use sqlx::{SqliteConnection, SqlitePool};

use crate::{journalist_id_key_queries, provisioning_key_queries, sentinel_id_key_queries};

/// Trait that unifies operations on journalist and sentinel ID key pairs
/// across the vault's database layer, with functions to manage candidate keys'
/// promotion to published keys.
pub trait PromotableIdKeyPair: Sized + PublicKey {
    type Unregistered;
    type SignedWithEpoch;
    type VerifiedPublicKey;

    #[allow(async_fn_in_trait)]
    async fn get_published_keys(
        conn: &mut SqliteConnection,
        now: DateTime<Utc>,
        trust_anchors: AnchorOrganizationPublicKeys,
    ) -> anyhow::Result<Vec<Self>>;

    fn into_latest(key_pairs: Vec<Self>) -> Option<Self>;

    fn get_epoch(signed_with_epoch: &Self::SignedWithEpoch) -> Epoch;

    /// Try to verify the untrusted signed key using a provisioning public key.
    /// Returns the verified public key if this provisioning key is the correct signer.
    fn try_verify(
        signed_with_epoch: &Self::SignedWithEpoch,
        provisioning_pk: &JournalistProvisioningPublicKey,
        now: DateTime<Utc>,
    ) -> Option<Self::VerifiedPublicKey>;

    /// Construct a registered (signed) key pair from the verified public key
    /// and the candidate's secret key.
    fn construct_registered_from_candidate(
        verified_pk: Self::VerifiedPublicKey,
        candidate: Self::Unregistered,
    ) -> Self;

    #[allow(async_fn_in_trait)]
    async fn get_or_create_candidate(
        conn: &mut SqliteConnection,
        now: DateTime<Utc>,
    ) -> anyhow::Result<(Self::Unregistered, DateTime<Utc>)>;

    #[allow(async_fn_in_trait)]
    async fn delete_candidate(
        conn: &mut SqliteConnection,
        candidate: &Self::Unregistered,
    ) -> anyhow::Result<()>;

    #[allow(async_fn_in_trait)]
    async fn insert_registered(
        conn: &mut SqliteConnection,
        provisioning_pk_id: i64,
        registered: &Self,
        created_at: DateTime<Utc>,
        published_at: DateTime<Utc>,
        epoch: Epoch,
    ) -> anyhow::Result<()>;

    #[allow(async_fn_in_trait)]
    async fn promote_candidate(
        pool: &SqlitePool,
        trust_anchors: AnchorOrganizationPublicKeys,
        candidate: Self::Unregistered,
        candidate_created_at: DateTime<Utc>,
        signed_with_epoch: Self::SignedWithEpoch,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let epoch = Self::get_epoch(&signed_with_epoch);
        let mut tx = pool.begin().await?;

        tracing::info!("Finding provisioning key");
        let maybe_provisioning_pk_and_verified =
            provisioning_key_queries::journalist_provisioning_pks(&mut tx, now, trust_anchors)
                .await?
                .find_map(|row| {
                    Self::try_verify(&signed_with_epoch, &row.pk, now)
                        .map(|verified_pk| (row.pk, verified_pk))
                });

        if let Some((provisioning_pk, verified_pk)) = maybe_provisioning_pk_and_verified {
            tracing::info!("Found provisioning key, deleting existing candidate key pair");

            Self::delete_candidate(&mut tx, &candidate).await?;

            tracing::info!("Getting provisioning public key ID");

            let provisioning_pk_id =
                provisioning_key_queries::journalist_provisioning_pk_id_from_pk(
                    &mut tx,
                    &provisioning_pk,
                )
                .await?
                .ok_or_else(|| {
                    anyhow::anyhow!("Provisioning key does not exist in journalist vault")
                })?;

            let registered = Self::construct_registered_from_candidate(verified_pk, candidate);

            tracing::info!(
                "Inserting registered ID key pair: {}",
                registered.public_key_hex()
            );

            let published_at = now;
            Self::insert_registered(
                &mut tx,
                provisioning_pk_id,
                &registered,
                candidate_created_at,
                published_at,
                epoch,
            )
            .await?;
        } else {
            anyhow::bail!(
                "Failed to find parent provisioning public key while inserting new identity key pair"
            );
        }

        tx.commit().await?;

        Ok(())
    }

    #[allow(async_fn_in_trait)]
    async fn last_published_at(
        conn: &mut SqliteConnection,
    ) -> anyhow::Result<Option<DateTime<Utc>>>;
}

impl PromotableIdKeyPair for JournalistIdKeyPair {
    type Unregistered = UnregisteredJournalistIdKeyPair;
    type SignedWithEpoch = UntrustedJournalistIdPublicKeyWithEpoch;
    type VerifiedPublicKey = JournalistIdPublicKey;

    async fn get_published_keys(
        conn: &mut SqliteConnection,
        now: DateTime<Utc>,
        trust_anchors: AnchorOrganizationPublicKeys,
    ) -> anyhow::Result<Vec<Self>> {
        let id_key_pairs =
            journalist_id_key_queries::published_journalist_id_key_pairs(conn, now, trust_anchors)
                .await?
                .map(|row| row.key_pair)
                .collect();
        Ok(id_key_pairs)
    }

    fn into_latest(key_pairs: Vec<Self>) -> Option<Self> {
        key_pairs.into_latest_key()
    }

    fn get_epoch(signed_with_epoch: &Self::SignedWithEpoch) -> Epoch {
        signed_with_epoch.epoch
    }

    async fn get_or_create_candidate(
        conn: &mut SqliteConnection,
        now: DateTime<Utc>,
    ) -> anyhow::Result<(Self::Unregistered, DateTime<Utc>)> {
        if let Some(row) = journalist_id_key_queries::candidate_journalist_id_key_pair(conn).await?
        {
            tracing::debug!("Found existing candidate journalist ID key pair");
            Ok((row.key_pair, row.added_at))
        } else {
            tracing::debug!("Generating new candidate journalist ID key pair");
            let candidate = UnsignedSigningKeyPair::generate();
            let added_at = now;
            journalist_id_key_queries::insert_candidate_journalist_id_key_pair(
                conn, &candidate, added_at,
            )
            .await?;
            Ok((candidate, added_at))
        }
    }

    fn try_verify(
        signed_with_epoch: &Self::SignedWithEpoch,
        provisioning_pk: &JournalistProvisioningPublicKey,
        now: DateTime<Utc>,
    ) -> Option<Self::VerifiedPublicKey> {
        signed_with_epoch.key.to_trusted(provisioning_pk, now).ok()
    }

    fn construct_registered_from_candidate(
        verified_pk: Self::VerifiedPublicKey,
        candidate: Self::Unregistered,
    ) -> Self {
        Self::new(verified_pk, candidate.secret_key)
    }

    async fn delete_candidate(
        conn: &mut SqliteConnection,
        candidate: &Self::Unregistered,
    ) -> anyhow::Result<()> {
        journalist_id_key_queries::delete_candidate_journalist_id_key_pair(conn, candidate).await?;
        Ok(())
    }

    async fn insert_registered(
        conn: &mut SqliteConnection,
        provisioning_pk_id: i64,
        registered: &Self,
        created_at: DateTime<Utc>,
        published_at: DateTime<Utc>,
        epoch: Epoch,
    ) -> anyhow::Result<()> {
        journalist_id_key_queries::insert_registered_journalist_id_key_pair(
            conn,
            provisioning_pk_id,
            registered,
            created_at,
            published_at,
            epoch,
        )
        .await
    }

    async fn last_published_at(
        conn: &mut SqliteConnection,
    ) -> anyhow::Result<Option<DateTime<Utc>>> {
        journalist_id_key_queries::last_published_journalist_id_key_pair_at(conn).await
    }
}

impl PromotableIdKeyPair for SentinelIdKeyPair {
    type Unregistered = UnregisteredSentinelIdKeyPair;
    type SignedWithEpoch = UntrustedSentinelIdPublicKeyWithEpoch;
    type VerifiedPublicKey = SentinelIdPublicKey;

    async fn get_published_keys(
        conn: &mut SqliteConnection,
        now: DateTime<Utc>,
        trust_anchors: AnchorOrganizationPublicKeys,
    ) -> anyhow::Result<Vec<Self>> {
        Ok(
            sentinel_id_key_queries::published_sentinel_id_key_pairs(conn, now, trust_anchors)
                .await?
                .map(|row| row.key_pair)
                .collect(),
        )
    }

    fn into_latest(key_pairs: Vec<Self>) -> Option<Self> {
        key_pairs.into_latest_key()
    }

    fn get_epoch(signed_with_epoch: &Self::SignedWithEpoch) -> Epoch {
        signed_with_epoch.epoch
    }

    async fn get_or_create_candidate(
        conn: &mut SqliteConnection,
        now: DateTime<Utc>,
    ) -> anyhow::Result<(Self::Unregistered, DateTime<Utc>)> {
        if let Some(row) = sentinel_id_key_queries::candidate_sentinel_id_key_pair(conn).await? {
            tracing::debug!("Found existing candidate sentinel id key pair");
            Ok((row.key_pair, row.added_at))
        } else {
            tracing::debug!("Generating new candidate sentinel id key pair");
            let candidate = UnsignedSigningKeyPair::generate();
            let added_at = now;
            sentinel_id_key_queries::insert_candidate_sentinel_id_key_pair(
                conn, &candidate, added_at,
            )
            .await?;
            Ok((candidate, added_at))
        }
    }

    fn try_verify(
        signed_with_epoch: &Self::SignedWithEpoch,
        provisioning_pk: &JournalistProvisioningPublicKey,
        now: DateTime<Utc>,
    ) -> Option<Self::VerifiedPublicKey> {
        signed_with_epoch.key.to_trusted(provisioning_pk, now).ok()
    }

    fn construct_registered_from_candidate(
        verified_pk: Self::VerifiedPublicKey,
        candidate: Self::Unregistered,
    ) -> Self {
        Self::new(verified_pk, candidate.secret_key)
    }

    async fn delete_candidate(
        conn: &mut SqliteConnection,
        candidate: &Self::Unregistered,
    ) -> anyhow::Result<()> {
        sentinel_id_key_queries::delete_candidate_sentinel_id_key_pair(conn, candidate).await?;
        Ok(())
    }

    async fn insert_registered(
        conn: &mut SqliteConnection,
        provisioning_pk_id: i64,
        registered: &Self,
        created_at: DateTime<Utc>,
        published_at: DateTime<Utc>,
        epoch: Epoch,
    ) -> anyhow::Result<()> {
        sentinel_id_key_queries::insert_registered_sentinel_id_key_pair(
            conn,
            provisioning_pk_id,
            registered,
            created_at,
            published_at,
            epoch,
        )
        .await
    }

    async fn last_published_at(
        conn: &mut SqliteConnection,
    ) -> anyhow::Result<Option<DateTime<Utc>>> {
        sentinel_id_key_queries::last_published_sentinel_id_key_pair_at(conn).await
    }
}
