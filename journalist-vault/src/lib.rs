mod backup_queries;
pub mod group_messaging;
mod group_messaging_queries;
mod info_queries;
mod journalist_id_key_queries;
pub mod key_rows;
pub mod logging;
mod message_queries;
mod msg_key_queries;
pub mod promotable_id_key_pair;
pub mod provisioning_key_queries;
mod sentinel_id_key_queries;
#[cfg(test)]
mod test_vault_clean_up;
mod user_queries;
mod vault_message;
pub mod vault_setup_bundle;

use anyhow::Context;
use key_rows::{
    AllVaultKeys, UntrustedCandidateJournalistIdKeyPairRow,
    UntrustedCandidateJournalistMessagingKeyPairRow, UntrustedCandidateSentinelIdKeyPairRow,
    UntrustedJournalistProvisioningPublicKeyRow, UntrustedPublishedJournalistIdKeyPairRow,
    UntrustedPublishedJournalistMessagingKeyPairRow, UntrustedPublishedSentinelIdKeyPairRow,
};
use std::path::Path;
pub use vault_message::{J2UMessage, U2JMessage, VaultMessage};

use crate::key_rows::SeedInfoRow;
use crate::logging::LoggingSession;
use crate::promotable_id_key_pair::PromotableIdKeyPair;
pub use backup_queries::BackupHistoryEntry;
use chrono::{DateTime, Utc};
use common::clap::Stage;
use common::{
    api::{
        forms::{
            PostJournalistForm, PostJournalistIdPublicKeyForm, PostSentinelIdPublicKeyForm,
            PostSentinelProfileForm,
        },
        models::{
            dead_drops::DeadDropId,
            journalist_id::JournalistIdentity,
            messages::{
                journalist_to_covernode_message::EncryptedJournalistToCoverNodeMessage,
                user_to_journalist_message_with_metadata::U2JMessageWithMetadata,
            },
            sentinel_id::SentinelIdentity,
        },
    },
    argon2_sqlcipher::Argon2SqlCipher,
    client::mailbox::mailbox_message::UserStatus,
    crypto::keys::{public_key::PublicKey, signing::SignedPublicSigningKey},
    epoch::Epoch,
    protocol::{
        constants::MESSAGE_VALID_FOR_DURATION,
        keys::{
            generate_journalist_messaging_key_pair, verify_journalist_provisioning_pk,
            AnchorOrganizationPublicKey, AnchorOrganizationPublicKeys, JournalistIdKeyPair,
            JournalistMessagingKeyPair, JournalistProvisioningPublicKey, LatestKey,
            SentinelIdKeyPair, UserPublicKey,
        },
        roles::JournalistProvisioning,
    },
    FixedSizeMessageText,
};
pub use group_messaging::{
    GroupId, GroupMessage, GroupMessageContent, GroupWithMessages, MessageId,
};
use logging::LogEntry;
use msg_key_queries::{
    candidate_msg_key_pair, insert_candidate_msg_key_pair,
    promote_candidate_msg_key_pair_to_published,
};
use sqlx::migrate::Migrate;
use sqlx::Acquire;
use sqlx::SqlitePool;
use thiserror::Error;
use tracing::info;

/// Errors that can occur when opening a vault.
/// These are exposed to the user interface to provide helpful error messages.
#[derive(Debug, Error)]
pub enum OpenVaultError {
    #[error("Path to vault does not exist: {0}")]
    PathNotFound(String),

    #[error("Incorrect password or database corrupted")]
    WrongPassword,

    #[error("Database migration failed. Your version of Sentinel may be outdated. Details: {0}")]
    MigrationFailed(String),

    #[error("Vault was created for stage {stored_stage:?} but you are trying to open it in stage {requested_stage:?}. Please select the correct environment for this vault.")]
    StageMismatch {
        stored_stage: Stage,
        requested_stage: Stage,
    },

    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

pub const VAULT_EXTENSION: &str = "vault";
pub const PASSWORD_EXTENSION: &str = "password";

pub type QueuedMessageId = i64;

pub struct EncryptedJournalistToCoverNodeMessageWithId {
    pub id: QueuedMessageId,
    pub message: EncryptedJournalistToCoverNodeMessage,
    pub deduplication_id: MessageId,
}

/// Some journalist vault functions can optionally replace an existing item. For example,
/// when adding a setup bundle you can either keep or replace the existing set up bundle.
///
/// This enum makes it clearer at the call site what is happening.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplacementStrategy {
    /// Keep an existing item
    Keep,
    /// Replace the existing item
    Replace,
}

pub struct User {
    pub user_pk: UserPublicKey,
    pub alias: Option<String>,
    pub description: Option<String>,
    pub status: UserStatus,
    pub marked_as_unread: bool,
}

pub struct GroupRow {
    pub id: GroupId,
    pub display_name: Option<String>,
    pub description: Option<String>,
}

#[derive(Clone)]
pub struct JournalistVault {
    pub pool: SqlitePool,
    trust_anchors: Vec<AnchorOrganizationPublicKey>,
}
impl JournalistVault {
    pub async fn create(
        path: impl AsRef<Path>,
        password: &str,
        journalist_id: &JournalistIdentity,
        sentinel_id: &SentinelIdentity,
        journalist_provisioning_pks: &[JournalistProvisioningPublicKey],
        now: DateTime<Utc>,
        stage: Stage,
    ) -> anyhow::Result<Self> {
        let trust_anchors = trust_anchors::get_trust_anchors(&stage, now)?;
        Self::internal_create(
            path,
            password,
            journalist_id,
            sentinel_id,
            journalist_provisioning_pks,
            now,
            trust_anchors,
            Some(stage),
        )
        .await
    }

    /// Creates a vault without setting a stage in vault_info.
    /// This simulates a vault created before the stage column was added,
    /// for testing the migration backfill path.
    #[cfg(feature = "test-utils")]
    pub async fn create_without_stage(
        path: impl AsRef<Path>,
        password: &str,
        journalist_id: &JournalistIdentity,
        sentinel_id: &SentinelIdentity,
        journalist_provisioning_pks: &[JournalistProvisioningPublicKey],
        now: DateTime<Utc>,
        trust_anchors: Vec<AnchorOrganizationPublicKey>,
    ) -> anyhow::Result<Self> {
        Self::internal_create(
            path,
            password,
            journalist_id,
            sentinel_id,
            journalist_provisioning_pks,
            now,
            trust_anchors,
            None,
        )
        .await
    }

    /// Creates a vault with explicitly provided trust anchors instead of deriving them from stage.
    /// This is useful for tests that generate their own organization key pairs.
    #[cfg(feature = "test-utils")]
    pub async fn create_with_trust_anchors(
        path: impl AsRef<Path>,
        password: &str,
        journalist_id: &JournalistIdentity,
        sentinel_id: &SentinelIdentity,
        journalist_provisioning_pks: &[JournalistProvisioningPublicKey],
        now: DateTime<Utc>,
        trust_anchors: Vec<AnchorOrganizationPublicKey>,
        stage: Stage,
    ) -> anyhow::Result<Self> {
        Self::internal_create(
            path,
            password,
            journalist_id,
            sentinel_id,
            journalist_provisioning_pks,
            now,
            trust_anchors,
            Some(stage),
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    async fn internal_create(
        path: impl AsRef<Path>,
        password: &str,
        journalist_id: &JournalistIdentity,
        sentinel_id: &SentinelIdentity,
        journalist_provisioning_pks: &[JournalistProvisioningPublicKey],
        now: DateTime<Utc>,
        trust_anchors: Vec<AnchorOrganizationPublicKey>,
        stage: Option<Stage>,
    ) -> anyhow::Result<Self> {
        let db = Argon2SqlCipher::new(path, password).await?;
        let pool = db.into_sqlite_pool();

        sqlx::migrate!().run(&pool).await?;

        let mut conn = pool.acquire().await?;

        info_queries::create_initial_info(&mut conn, journalist_id, sentinel_id, stage).await?;

        for journalist_provisioning_pk in journalist_provisioning_pks {
            provisioning_key_queries::insert_journalist_provisioning_pk(
                &mut conn,
                journalist_provisioning_pk,
                now,
            )
            .await?;
        }

        Ok(Self {
            pool,
            trust_anchors,
        })
    }

    pub async fn open(
        path: impl AsRef<Path>,
        password: &str,
        stage: Stage,
    ) -> Result<Self, OpenVaultError> {
        let trust_anchors = trust_anchors::get_trust_anchors(&stage, Utc::now())?;
        Self::internal_open(path, password, trust_anchors, stage).await
    }

    /// Opens a vault with explicitly provided trust anchors instead of deriving them from stage.
    /// This is useful for tests that generate their own organization key pairs.
    #[cfg(feature = "test-utils")]
    pub async fn open_with_trust_anchors(
        path: impl AsRef<Path>,
        password: &str,
        trust_anchors: Vec<AnchorOrganizationPublicKey>,
        stage: Stage,
    ) -> Result<Self, OpenVaultError> {
        Self::internal_open(path, password, trust_anchors, stage).await
    }

    async fn internal_open(
        path: impl AsRef<Path>,
        password: &str,
        trust_anchors: Vec<AnchorOrganizationPublicKey>,
        stage: Stage,
    ) -> Result<Self, OpenVaultError> {
        if !path.as_ref().exists() {
            return Err(OpenVaultError::PathNotFound(
                path.as_ref().display().to_string(),
            ));
        }

        let db = Argon2SqlCipher::open_and_maybe_migrate_from_legacy(&path, password)
            .await
            .map_err(|e| {
                tracing::debug!("Failed to open database: {e}");
                OpenVaultError::WrongPassword
            })?;
        let pool = db.into_sqlite_pool();

        sqlx::migrate!()
            .run(&pool)
            .await
            .map_err(|e| OpenVaultError::MigrationFailed(e.to_string()))?;

        // Attempt to read out the journalist ID - if the encryption key is wrong this will fail
        let mut conn = pool
            .acquire()
            .await
            .map_err(|e| OpenVaultError::Other(anyhow::Error::from(e)))?;
        let _ = info_queries::journalist_id(&mut conn)
            .await
            .map_err(OpenVaultError::Other)?;

        // Check the stage stored in the vault.
        // If the vault was created before the stage column was added, backfill it with the current stage.
        // If the stage is already set and doesn't match, fail the open.
        match info_queries::stage(&mut conn)
            .await
            .map_err(OpenVaultError::Other)?
        {
            Some(stored_stage) if stored_stage != stage => {
                return Err(OpenVaultError::StageMismatch {
                    stored_stage,
                    requested_stage: stage,
                });
            }
            Some(_) => {
                // Stage matches, all good
            }
            None => {
                // Backfill the stage for vaults that were created before the stage column was added
                info!("The vault has no stage set. Backfilling the stage with the current stage {:?}.", stage);
                info_queries::set_stage(&mut conn, stage)
                    .await
                    .map_err(OpenVaultError::Other)?;

                // TODO: remove after 2026-06-01 when all clients should have been migrated
                // See: https://github.com/guardian/coverdrop-internal/issues/3910
            }
        }

        Ok(Self {
            pool,
            trust_anchors,
        })
    }

    pub async fn check_password(&self, path: impl AsRef<Path>, password: &str) -> bool {
        Argon2SqlCipher::check_can_open_database_assuming_migrated(path, password).await
    }

    /// Changes the password of the vault.
    pub async fn change_password(&self, new_password: &str) -> anyhow::Result<()> {
        let mut conn = self.pool.acquire().await?;

        Argon2SqlCipher::rekey_database(&mut conn, new_password).await?;

        Ok(())
    }

    //
    // Logging
    //

    pub async fn add_session(&self, session_started_at: DateTime<Utc>) -> anyhow::Result<i64> {
        let mut conn = self.pool.acquire().await?;
        logging::insert_session(&mut conn, session_started_at).await
    }

    pub async fn add_log_entries(
        &self,
        session_id: i64,
        log_entry: &[LogEntry],
    ) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;

        logging::insert_log_entries(&mut tx, session_id, log_entry).await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn get_log_session_timeline(&self) -> anyhow::Result<Vec<LoggingSession>> {
        let mut conn = self.pool.acquire().await?;
        logging::get_session_timeline(&mut conn).await
    }

    pub async fn get_log_entries(
        &self,
        min_level: String,
        search_term: String,
        before: DateTime<Utc>,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<LogEntry>> {
        let mut conn = self.pool.acquire().await?;
        logging::select_log_entries(&mut conn, min_level, search_term, before, limit, offset).await
    }

    //
    // Info
    //

    pub async fn journalist_id(&self) -> anyhow::Result<JournalistIdentity> {
        let mut conn = self.pool.acquire().await?;
        info_queries::journalist_id(&mut conn).await
    }

    pub async fn last_published_msg_key_pair_at(&self) -> anyhow::Result<Option<DateTime<Utc>>> {
        let mut conn = self.pool.acquire().await?;
        msg_key_queries::last_published_msg_key_pair_at(&mut conn).await
    }

    pub async fn max_dead_drop_id(&self) -> anyhow::Result<DeadDropId> {
        let mut conn = self.pool.acquire().await?;
        info_queries::max_dead_drop_id(&mut conn).await
    }

    pub async fn all_vault_keys(&self, now: DateTime<Utc>) -> anyhow::Result<AllVaultKeys> {
        let mut conn = self.pool.begin().await?;

        let trust_anchors = self.trust_anchors()?;
        let org_pks_untrusted = trust_anchors.to_untrusted();

        let journalist_provisioning_pks = provisioning_key_queries::journalist_provisioning_pks(
            &mut conn,
            now,
            trust_anchors.clone(),
        )
        .await?
        .map(|row| UntrustedJournalistProvisioningPublicKeyRow {
            id: row.id,
            pk: row.pk.to_untrusted(),
        })
        .collect();

        let published_sentinel_id_key_pairs =
            sentinel_id_key_queries::published_sentinel_id_key_pairs(
                &mut conn,
                now,
                trust_anchors.clone(),
            )
            .await?
            .map(|row| UntrustedPublishedSentinelIdKeyPairRow {
                id: row.id,
                key_pair: row.key_pair.to_untrusted(),
                epoch: row.epoch,
            })
            .collect();

        let published_journalist_id_key_pairs =
            journalist_id_key_queries::published_journalist_id_key_pairs(
                &mut conn,
                now,
                trust_anchors.clone(),
            )
            .await?
            .map(|row| UntrustedPublishedJournalistIdKeyPairRow {
                id: row.id,
                key_pair: row.key_pair.to_untrusted(),
                epoch: row.epoch,
            })
            .collect();

        let published_msg_key_pairs =
            msg_key_queries::published_msg_key_pairs(&mut conn, now, trust_anchors.clone())
                .await?
                .map(|row| UntrustedPublishedJournalistMessagingKeyPairRow {
                    id: row.id,
                    key_pair: row.key_pair.to_untrusted(),
                    epoch: row.epoch,
                })
                .collect();

        let candidate_sentinel_id_key_pair =
            sentinel_id_key_queries::candidate_sentinel_id_key_pair(&mut conn)
                .await?
                .map(|row| UntrustedCandidateSentinelIdKeyPairRow {
                    id: row.id,
                    added_at: row.added_at,
                    key_pair: row.key_pair.to_untrusted(),
                });

        let candidate_journalist_id_key_pair =
            journalist_id_key_queries::candidate_journalist_id_key_pair(&mut conn)
                .await?
                .map(|row| UntrustedCandidateJournalistIdKeyPairRow {
                    id: row.id,
                    added_at: row.added_at,
                    key_pair: row.key_pair.to_untrusted(),
                });

        let candidate_msg_key_pair =
            msg_key_queries::candidate_msg_key_pair(&mut conn, now, trust_anchors.clone())
                .await?
                .map(|row| UntrustedCandidateJournalistMessagingKeyPairRow {
                    id: row.id,
                    added_at: row.added_at,
                    key_pair: row.key_pair.to_untrusted(),
                });

        Ok(AllVaultKeys {
            org_pks: org_pks_untrusted,
            journalist_provisioning_pks,
            candidate_sentinel_id_key_pair,
            published_sentinel_id_key_pairs,
            candidate_journalist_id_key_pair,
            published_journalist_id_key_pairs,
            candidate_msg_key_pair,
            published_msg_key_pairs,
        })
    }

    //
    // Group messaging
    //
    // The following functions take an SqliteConnection so that they can be used in transactions.
    // This avoids partial failures when updates need to be made to both these tables
    // and OpenMLS tables via the VaultProvider.
    //

    pub async fn insert_group(
        &self,
        conn: &mut sqlx::SqliteConnection,
        id: &GroupId,
        display_name: Option<&str>,
        description: Option<&str>,
    ) -> anyhow::Result<()> {
        group_messaging_queries::insert_group(conn, id, display_name, description).await
    }

    pub async fn update_group(
        &self,
        conn: &mut sqlx::SqliteConnection,
        id: &GroupId,
        display_name: &String,
        description: &String,
    ) -> anyhow::Result<()> {
        group_messaging_queries::update_group(conn, id, display_name, description).await
    }

    pub async fn update_group_name(
        &self,
        conn: &mut sqlx::SqliteConnection,
        id: &GroupId,
        display_name: &str,
    ) -> anyhow::Result<()> {
        group_messaging_queries::update_group_name(conn, id, display_name).await
    }

    pub async fn update_group_description(
        &self,
        conn: &mut sqlx::SqliteConnection,
        id: &GroupId,
        description: &String,
    ) -> anyhow::Result<()> {
        group_messaging_queries::update_group_description(conn, id, description).await
    }

    pub async fn get_group(&self, id: &GroupId) -> anyhow::Result<GroupRow> {
        let mut conn = self.pool.acquire().await?;
        group_messaging_queries::get_group(&mut conn, id).await
    }

    pub async fn get_group_with_messages(&self, id: &GroupId) -> anyhow::Result<GroupWithMessages> {
        let mut conn = self.pool.acquire().await?;
        group_messaging_queries::get_group_with_messages(&mut conn, id).await
    }

    pub async fn get_group_with_connection(
        &self,
        conn: &mut sqlx::SqliteConnection,
        id: &GroupId,
    ) -> anyhow::Result<GroupRow> {
        group_messaging_queries::get_group(conn, id).await
    }

    pub async fn get_groups(&self) -> anyhow::Result<Vec<GroupRow>> {
        let mut conn = self.pool.acquire().await?;
        group_messaging_queries::get_groups(&mut conn).await
    }

    pub async fn insert_j2j_message(
        &self,
        conn: &mut sqlx::SqliteConnection,
        message: &GroupMessage,
    ) -> anyhow::Result<()> {
        group_messaging_queries::insert_j2j_message(conn, message).await
    }

    pub async fn update_group_messages_read_status(
        &self,
        group_id: &GroupId,
        message_ids: &[MessageId],
        read: bool,
    ) -> anyhow::Result<()> {
        let mut conn = self.pool.acquire().await?;
        group_messaging_queries::update_group_messages_read_status(
            &mut conn,
            group_id,
            message_ids,
            read,
        )
        .await
    }

    pub async fn get_groups_and_messages(&self) -> anyhow::Result<Vec<GroupWithMessages>> {
        let mut conn = self.pool.acquire().await?;
        group_messaging_queries::get_groups_and_messages(&mut conn).await
    }

    //
    // Users
    //

    pub async fn users(&self) -> anyhow::Result<Vec<User>> {
        let mut conn = self.pool.acquire().await?;

        user_queries::users(&mut conn).await
    }

    pub async fn update_user_alias_and_description(
        &self,
        user_pk: &UserPublicKey,
        alias: &str,
        description: &str,
    ) -> anyhow::Result<()> {
        let mut conn = self.pool.acquire().await?;

        user_queries::update_user_alias_and_description(&mut conn, user_pk, alias, description)
            .await
    }

    //
    // Messages
    //

    pub async fn add_messages_from_user_to_journalist_and_update_max_dead_drop_id(
        &self,
        messages: &[U2JMessageWithMetadata],
        max_dead_drop_id: i32,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;

        for message in messages {
            // insert into users table if not already present
            user_queries::add_user(&mut tx, &message.u2j_message.reply_key, now).await?;

            message_queries::add_u2j_message(
                &mut tx,
                &message.u2j_message.reply_key,
                &message.u2j_message.message,
                now,
                message.unsigned_dead_drop_id,
                message.dead_drop_created_at,
            )
            .await?;
        }

        info_queries::set_max_dead_drop_id(&mut tx, max_dead_drop_id).await?;

        tx.commit().await?;

        Ok(())
    }

    // TODO:
    // This should be the only option. The two partial versions of this are not
    // transaction safe.
    pub async fn add_message_from_journalist_to_user_and_enqueue(
        &self,
        user_pk: &UserPublicKey,
        // Maybe passing in both the encrypted and unencrypted is a bit weird but otherwise
        // we'd be passing in the public key hierarchy.
        unencrypted_message: &FixedSizeMessageText,
        encrypted_message: EncryptedJournalistToCoverNodeMessage,
        deduplication_id: MessageId,
        now: DateTime<Utc>,
    ) -> anyhow::Result<i64> {
        let mut tx = self.pool.begin().await?;

        // insert into users table if not already present
        user_queries::add_user(&mut tx, user_pk, now).await?;

        let queue_id =
            message_queries::enqueue_message(&mut tx, encrypted_message, deduplication_id).await?;
        message_queries::add_j2u_message(
            &mut tx,
            user_pk,
            unencrypted_message,
            now,
            Some(queue_id),
        )
        .await?;

        let queue_length = message_queries::get_queue_length(&mut tx).await?;

        tx.commit().await?;

        Ok(queue_length)
    }

    /// Get the oldest message in a journalist's outbound queue
    pub async fn head_queue_message(
        &self,
    ) -> anyhow::Result<Option<EncryptedJournalistToCoverNodeMessageWithId>> {
        let mut conn = self.pool.acquire().await?;
        message_queries::peek_head_queue_message(&mut conn).await
    }

    /// Delete a message from the outbound queue and returns the new queue length.
    /// This should only be called after a message has successfully been sent to the Kinesis stream
    pub async fn delete_queue_message(&self, id: i64) -> anyhow::Result<i64> {
        let mut tx = self.pool.begin().await?;

        message_queries::delete_queue_message(&mut tx, id).await?;

        let new_queue_length = message_queries::get_queue_length(&mut tx).await?;

        tx.commit().await?;

        Ok(new_queue_length)
    }

    pub async fn mark_as_read(&self, user_pk: &UserPublicKey) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;
        message_queries::mark_as_read(&mut tx, user_pk).await?;
        user_queries::mark_as_read(&mut tx, user_pk).await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn mark_as_unread(&self, user_pk: &UserPublicKey) -> anyhow::Result<()> {
        let mut conn = self.pool.acquire().await?;
        user_queries::mark_as_unread(&mut conn, user_pk).await
    }

    pub async fn set_custom_expiry(
        &self,
        message: &VaultMessage,
        custom_expiry: Option<DateTime<Utc>>,
    ) -> anyhow::Result<()> {
        let mut conn = self.pool.acquire().await?;
        message_queries::set_custom_expiry(&mut conn, message, custom_expiry).await
    }

    pub async fn messages(&self) -> anyhow::Result<Vec<VaultMessage>> {
        let mut conn = self.pool.acquire().await?;

        message_queries::messages(&mut conn).await
    }

    pub async fn update_user_status(
        &self,
        user_pk: &UserPublicKey,
        status: UserStatus,
    ) -> anyhow::Result<()> {
        let mut conn = self.pool.acquire().await?;

        user_queries::update_user_status(&mut conn, user_pk, status).await
    }

    //
    // Keys
    //

    // TODO move toward using trust_anchors() and AnchorOrganizationPublicKeys
    pub fn org_pks(&self) -> anyhow::Result<Vec<AnchorOrganizationPublicKey>> {
        Ok(self.trust_anchors.clone())
    }

    pub fn trust_anchors(&self) -> anyhow::Result<AnchorOrganizationPublicKeys> {
        Ok(AnchorOrganizationPublicKeys::new(
            self.trust_anchors.clone(),
        ))
    }

    pub async fn provisioning_pks(
        &self,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Vec<JournalistProvisioningPublicKey>> {
        let mut conn = self.pool.acquire().await?;

        let trust_anchors = self.trust_anchors()?;
        let provisioning_keys =
            provisioning_key_queries::journalist_provisioning_pks(&mut conn, now, trust_anchors)
                .await?
                .map(|row| row.pk)
                .collect();

        Ok(provisioning_keys)
    }

    pub async fn add_provisioning_pk(
        &self,
        journalist_provisioning_pk: &JournalistProvisioningPublicKey,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let mut conn = self.pool.acquire().await?;

        provisioning_key_queries::insert_journalist_provisioning_pk(
            &mut conn,
            journalist_provisioning_pk,
            now,
        )
        .await
    }

    pub async fn journalist_id_key_pairs(
        &self,
        now: DateTime<Utc>,
    ) -> anyhow::Result<impl Iterator<Item = JournalistIdKeyPair>> {
        let mut conn = self.pool.acquire().await?;

        let trust_anchors = self.trust_anchors()?;
        let id_key_pairs = journalist_id_key_queries::published_journalist_id_key_pairs(
            &mut conn,
            now,
            trust_anchors,
        )
        .await?
        .map(|row| row.key_pair);

        Ok(id_key_pairs)
    }

    pub async fn latest_journalist_id_key_pair(
        &self,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Option<JournalistIdKeyPair>> {
        self.latest_id_key_pair::<JournalistIdKeyPair>(now).await
    }

    pub async fn latest_id_key_pair<KP: PromotableIdKeyPair>(
        &self,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Option<KP>> {
        let mut conn = self.pool.acquire().await?;
        let trust_anchors = self.trust_anchors()?;
        let key_pairs = KP::get_published_keys(&mut conn, now, trust_anchors).await?;
        Ok(KP::into_latest(key_pairs))
    }

    pub async fn get_or_create_candidate_id_key_pair<KP: PromotableIdKeyPair>(
        &self,
        now: DateTime<Utc>,
    ) -> anyhow::Result<(KP::Unregistered, DateTime<Utc>)> {
        let mut conn = self.pool.acquire().await?;
        KP::get_or_create_candidate(&mut conn, now).await
    }

    pub async fn promote_candidate_id_key_pair<KP: PromotableIdKeyPair>(
        &self,
        candidate: KP::Unregistered,
        candidate_created_at: DateTime<Utc>,
        signed_with_epoch: KP::SignedWithEpoch,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let trust_anchors = self.trust_anchors()?;
        KP::promote_candidate(
            &self.pool,
            trust_anchors,
            candidate,
            candidate_created_at,
            signed_with_epoch,
            now,
        )
        .await
    }

    pub async fn last_published_id_key_pair_at<KP: PromotableIdKeyPair>(
        &self,
    ) -> anyhow::Result<Option<DateTime<Utc>>> {
        let mut conn = self.pool.acquire().await?;
        KP::last_published_at(&mut conn).await
    }

    pub async fn msg_key_pairs_for_decryption(
        &self,
        now: DateTime<Utc>,
    ) -> anyhow::Result<impl Iterator<Item = JournalistMessagingKeyPair>> {
        let mut conn = self.pool.acquire().await?;

        let trust_anchors = self.trust_anchors()?;
        let candidate_msg_key_pair =
            msg_key_queries::candidate_msg_key_pair(&mut conn, now, trust_anchors.clone())
                .await?
                .into_iter()
                .map(|row| row.key_pair);

        let published_msg_key_pairs =
            msg_key_queries::published_msg_key_pairs(&mut conn, now, trust_anchors)
                .await?
                .map(|iter| iter.key_pair);

        let combined_msg_key_pairs = candidate_msg_key_pair.chain(published_msg_key_pairs);

        Ok(combined_msg_key_pairs)
    }

    pub async fn latest_msg_key_pair(
        &self,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Option<JournalistMessagingKeyPair>> {
        let mut conn = self.pool.acquire().await?;

        let trust_anchors = self.trust_anchors()?;
        let latest_key_pair =
            msg_key_queries::published_msg_key_pairs(&mut conn, now, trust_anchors)
                .await?
                .map(|key_pair_row| key_pair_row.key_pair)
                .collect::<Vec<_>>()
                .into_latest_key();

        Ok(latest_key_pair)
    }

    pub async fn user_keys(&self) -> anyhow::Result<impl Iterator<Item = UserPublicKey> + '_> {
        let mut conn = self.pool.acquire().await?;

        user_queries::user_pks(&mut conn).await
    }

    /// Generates a new ID key pair and upload form, requires a journalist provisioning
    /// key pair. This is generally used when regular key rotation is impossible, such as
    /// when initially creating the vault (since there's no previous ID key pair) or when
    /// a journalist has not opened their app recently enough to allow rotation to happen
    /// and their ID key pairs have all expired.
    ///
    /// Since this requires a journalist provisioning key pair it is recommended that this
    /// is never done while connected to the internet or other untrusted network.
    #[allow(clippy::too_many_arguments)]
    pub async fn add_vault_setup_bundle(
        &self,
        journalist_provisioning_pk: &JournalistProvisioningPublicKey,
        journalist_id_key_pair: JournalistIdKeyPair,
        journalist_id_pk_upload_form: PostJournalistIdPublicKeyForm,
        register_journalist_form: Option<PostJournalistForm>,
        sentinel_id_pk_upload_form: Option<PostSentinelIdPublicKeyForm>,
        sentinel_id_key_pair: Option<&SentinelIdKeyPair>,
        register_sentinel_profile_form: Option<PostSentinelProfileForm>,
        replacement_strategy: ReplacementStrategy,
    ) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;
        tx.begin().await?;

        let Some(provisioning_pk_id) =
            provisioning_key_queries::journalist_provisioning_pk_id_from_pk(
                &mut tx,
                journalist_provisioning_pk,
            )
            .await?
        else {
            anyhow::bail!(
                "Journalist provisioning key provided to add_vault_setup_bundle was not found in the vault"
            );
        };

        vault_setup_bundle::insert_vault_setup_bundle(
            &mut tx,
            provisioning_pk_id,
            &journalist_id_key_pair,
            journalist_id_pk_upload_form,
            register_journalist_form,
            sentinel_id_pk_upload_form,
            sentinel_id_key_pair,
            register_sentinel_profile_form,
            replacement_strategy,
        )
        .await?;

        tx.commit().await?;

        Ok(())
    }

    /// Get the vault setup bundle if one exists.
    pub async fn get_vault_setup_bundle(
        &self,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Option<SeedInfoRow>> {
        let mut conn = self.pool.acquire().await?;
        let trust_anchors = self.trust_anchors()?;
        vault_setup_bundle::get_vault_setup_bundle(&mut *conn, now, trust_anchors).await
    }

    /// Delete the vault setup bundle.
    pub async fn delete_vault_setup_bundle(&self) -> anyhow::Result<()> {
        let mut conn = self.pool.acquire().await?;
        vault_setup_bundle::delete_vault_setup_bundle(&mut *conn).await
    }

    /// Insert a registered journalist ID key pair into the vault.
    pub async fn insert_registered_journalist_id_key_pair(
        &self,
        provisioning_pk_id: i64,
        id_key_pair: &JournalistIdKeyPair,
        created_at: DateTime<Utc>,
        published_at: DateTime<Utc>,
        epoch: Epoch,
    ) -> anyhow::Result<()> {
        let mut conn = self.pool.acquire().await?;
        journalist_id_key_queries::insert_registered_journalist_id_key_pair(
            &mut conn,
            provisioning_pk_id,
            id_key_pair,
            created_at,
            published_at,
            epoch,
        )
        .await
    }

    /// Insert a registered sentinel ID key pair into the vault.
    pub async fn insert_registered_sentinel_id_key_pair(
        &self,
        provisioning_pk_id: i64,
        id_key_pair: &SentinelIdKeyPair,
        created_at: DateTime<Utc>,
        published_at: DateTime<Utc>,
        epoch: Epoch,
    ) -> anyhow::Result<()> {
        let mut conn = self.pool.acquire().await?;
        sentinel_id_key_queries::insert_registered_sentinel_id_key_pair(
            &mut conn,
            provisioning_pk_id,
            id_key_pair,
            created_at,
            published_at,
            epoch,
        )
        .await
    }

    /// Set the max dead drop ID in the vault.
    pub async fn set_max_dead_drop_id(&self, max_dead_drop_id: i32) -> anyhow::Result<()> {
        let mut conn = self.pool.acquire().await?;
        info_queries::set_max_dead_drop_id(&mut conn, max_dead_drop_id).await
    }

    pub async fn max_delivery_service_message_id(&self) -> anyhow::Result<u32> {
        let mut conn = self.pool.acquire().await?;
        info_queries::max_delivery_service_message_id(&mut conn).await
    }

    pub async fn set_max_delivery_service_message_id(
        &self,
        conn: &mut sqlx::SqliteConnection,
        message_id: u32,
    ) -> anyhow::Result<()> {
        info_queries::set_max_delivery_service_message_id(conn, message_id).await
    }

    /// Takes an iterator of journalist provisioning keys and inserts any that aren't already in the vault
    /// after verifying them with trust anchors.
    pub async fn sync_journalist_provisioning_pks(
        &self,
        api_journalist_provisioning_pks: &Vec<&SignedPublicSigningKey<JournalistProvisioning>>,
        now: DateTime<Utc>,
    ) -> anyhow::Result<(), anyhow::Error> {
        let vault_journalist_provisioning_pks = self.provisioning_pks(now).await?;
        let journalist_provisioning_pks_to_insert: Vec<_> = api_journalist_provisioning_pks
            .iter()
            .filter(|key| !vault_journalist_provisioning_pks.contains(key))
            .collect();

        if journalist_provisioning_pks_to_insert.is_empty() {
            tracing::info!("No new provisioning keys from API to insert into vault");
            return Ok(());
        } else {
            tracing::info!(
                "Found {} new provisioning keys to add to vault",
                journalist_provisioning_pks_to_insert.len()
            )
        }

        let org_pks = self.org_pks()?;
        for journalist_provisioning_pk in journalist_provisioning_pks_to_insert {
            // find the trust anchor that has signed the provisioning key to insert
            let maybe_verified_journalist_provisioning_pk = org_pks.iter().find_map(|org_pk| {
                let org_pk = org_pk.to_non_anchor();
                verify_journalist_provisioning_pk(
                    &journalist_provisioning_pk.to_untrusted(),
                    &org_pk,
                    now,
                )
                .ok()
            });

            if let Some(journalist_provisioning_pk) = maybe_verified_journalist_provisioning_pk {
                tracing::info!(
                    "Found signing key for provisioning key. Inserting provisioning key."
                );
                self.add_provisioning_pk(&journalist_provisioning_pk, now)
                    .await?;
            } else {
                tracing::warn!(
                    "Could not find trust anchor for journalist provisioning public key {}",
                    journalist_provisioning_pk.public_key_hex()
                );
            };
        }

        Ok(())
    }

    /// Get the existing candidate messaging key pair, or create one if it doesn't exist.
    pub async fn get_or_create_candidate_msg_key_pair(
        &self,
        now: DateTime<Utc>,
    ) -> anyhow::Result<JournalistMessagingKeyPair> {
        let Some(latest_id_key_pair) = self.latest_journalist_id_key_pair(now).await? else {
            anyhow::bail!(
                "No ID key pairs present in vault, cannot create messaging key pair. Use a journalist provisioning key pair to create a new seed ID key pair."
            )
        };

        let mut conn = self.pool.acquire().await?;

        let trust_anchors = self.trust_anchors()?;
        let candidate_msg_key_pair = if let Some(candidate_msg_key_pair) =
            candidate_msg_key_pair(&mut conn, now, trust_anchors.clone()).await?
        {
            candidate_msg_key_pair.key_pair
        } else {
            let candidate_msg_key_pair =
                generate_journalist_messaging_key_pair(&latest_id_key_pair, now);

            insert_candidate_msg_key_pair(
                &mut conn,
                latest_id_key_pair.public_key(),
                &candidate_msg_key_pair,
                now,
                trust_anchors.clone(),
            )
            .await?;

            candidate_msg_key_pair
        };

        Ok(candidate_msg_key_pair)
    }

    /// Promote a candidate messaging key pair to published with the given epoch.
    pub async fn promote_candidate_msg_key_pair(
        &self,
        msg_key_pair: &JournalistMessagingKeyPair,
        epoch: Epoch,
    ) -> anyhow::Result<()> {
        let mut conn = self.pool.acquire().await?;
        promote_candidate_msg_key_pair_to_published(&mut conn, msg_key_pair, epoch).await
    }

    /// - Delete expired id and msg key pairs
    /// - Delete expired provisioning public keys
    /// - Remove messages that are more than MESSAGE_VALID_FOR_DURATION old
    /// - Delete old logs
    pub async fn clean_up(&self, now: DateTime<Utc>) -> anyhow::Result<()> {
        let message_deletion_duration = MESSAGE_VALID_FOR_DURATION;

        let mut tx = self.pool.begin().await?;

        message_queries::delete_messages_before(&mut tx, now, message_deletion_duration)
            .await
            .context("delete old messages")?;

        // Delete expired keys
        msg_key_queries::delete_expired_msg_key_pairs(&mut tx, now)
            .await
            .context("delete expired msg key pairs")?;
        journalist_id_key_queries::delete_expired_journalist_id_key_pairs(&mut tx, now)
            .await
            .context("delete expired id key pairs")?;
        sentinel_id_key_queries::delete_expired_sentinel_id_key_pairs(&mut tx, now)
            .await
            .context("delete expired sentinel id key pairs")?;
        provisioning_key_queries::delete_expired_provisioning_pks(&mut tx, now)
            .await
            .context("delete expired provisioning pks")?;

        logging::delete_old_logs(&mut tx, now)
            .await
            .context("delete old logs")?;

        tx.commit().await?;

        let mut conn = self.pool.acquire().await?;
        sqlx::query!("VACUUM")
            .execute(&mut *conn)
            .await
            .context("vacuuming")?;

        Ok(())
    }

    //
    // Backups
    //

    pub async fn record_manual_backup(
        &self,
        timestamp: DateTime<Utc>,
        path: &str,
    ) -> anyhow::Result<()> {
        let mut conn = self.pool.acquire().await?;

        backup_queries::record_manual_backup(&mut conn, timestamp, path).await
    }

    pub async fn record_automated_backup(
        &self,
        timestamp: DateTime<Utc>,
        recovery_contact_journalist_ids: Vec<JournalistIdentity>,
    ) -> anyhow::Result<()> {
        let mut conn = self.pool.acquire().await?;

        backup_queries::record_automated_backup(
            &mut conn,
            timestamp,
            recovery_contact_journalist_ids,
        )
        .await
    }

    pub async fn get_count_of_keys_created_since_last_backup(&self) -> anyhow::Result<i64> {
        let mut conn = self.pool.acquire().await?;

        backup_queries::get_count_of_keys_created_since_last_backup(&mut conn).await
    }

    pub async fn get_backup_contacts(&self) -> anyhow::Result<Vec<JournalistIdentity>> {
        let mut conn = self.pool.acquire().await?;

        backup_queries::get_backup_contacts(&mut conn).await
    }

    pub async fn set_backup_contacts(
        &self,
        contacts: Vec<JournalistIdentity>,
    ) -> anyhow::Result<()> {
        let mut conn = self.pool.acquire().await?;

        backup_queries::set_backup_contacts(&mut conn, contacts).await
    }

    pub async fn get_backup_history(&self) -> anyhow::Result<Vec<BackupHistoryEntry>> {
        let mut conn = self.pool.acquire().await?;

        backup_queries::get_backup_history(&mut conn).await
    }

    pub async fn remove_invalid_backup_contacts(
        &self,
        journalist_identities_from_api: Vec<&JournalistIdentity>,
    ) -> anyhow::Result<u64> {
        let mut conn = self.pool.acquire().await?;

        backup_queries::remove_invalid_backup_contacts(&mut conn, journalist_identities_from_api)
            .await
    }

    //
    // Sentinel identity
    //

    pub async fn sentinel_id(&self) -> anyhow::Result<Option<SentinelIdentity>> {
        let mut conn = self.pool.acquire().await?;
        info_queries::sentinel_id(&mut conn).await
    }

    pub async fn set_sentinel_id(&self, sentinel_id: &SentinelIdentity) -> anyhow::Result<()> {
        let mut conn = self.pool.acquire().await?;
        info_queries::set_sentinel_id(&mut conn, sentinel_id).await
    }

    //
    // Sentinel ID key pairs
    //

    pub async fn sentinel_id_key_pairs(
        &self,
        now: DateTime<Utc>,
    ) -> anyhow::Result<impl Iterator<Item = SentinelIdKeyPair>> {
        let mut conn = self.pool.acquire().await?;

        let trust_anchors = self.trust_anchors()?;
        let id_key_pairs =
            sentinel_id_key_queries::published_sentinel_id_key_pairs(&mut conn, now, trust_anchors)
                .await?
                .map(|row| row.key_pair);

        Ok(id_key_pairs)
    }

    pub async fn latest_sentinel_id_key_pair(
        &self,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Option<SentinelIdKeyPair>> {
        self.latest_id_key_pair::<SentinelIdKeyPair>(now).await
    }
}

/// Revert the last applied migration on the given pool.
/// Requires a `.down.sql` file to exist for the migration being reverted.
pub async fn revert_last_migration(pool: &SqlitePool) -> anyhow::Result<()> {
    use sqlx::migrate::MigrationType;

    let migrator = sqlx::migrate!();

    let applied: Vec<sqlx::migrate::AppliedMigration> =
        pool.acquire().await?.list_applied_migrations().await?;

    let mut versions: Vec<i64> = applied.iter().map(|m| m.version).collect();
    versions.sort();

    let target = match versions.len() {
        0 => anyhow::bail!("No migrations have been applied"),
        1 => 0,
        n => versions[n - 2],
    };

    let latest = *versions.last().unwrap();

    let has_down = migrator
        .iter()
        .any(|m| m.version == latest && m.migration_type == MigrationType::ReversibleDown);

    if !has_down {
        anyhow::bail!("Migration {latest} has no .down.sql file and cannot be reverted");
    }

    println!("Reverting migration version {latest} (target: {target})");

    migrator.undo(pool, target).await?;

    println!("Migration {latest} reverted successfully");
    Ok(())
}
