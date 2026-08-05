use anyhow::Context;
use chrono::{DateTime, Utc};
use common::{
    epoch::Epoch,
    protocol::keys::{
        verify_journalist_provisioning_pk, AnchorOrganizationPublicKeys, SentinelIdKeyPair,
        UnregisteredSentinelIdKeyPair, UntrustedJournalistProvisioningPublicKey,
        UntrustedSentinelIdKeyPair, UntrustedUnregisteredSentinelIdKeyPair,
    },
};
use sqlx::SqliteConnection;

use crate::key_rows::{
    CandidateKeyPairRow, CandidateSentinelIdKeyPairRow, PublishedSentinelIdKeyPairRow,
};

pub(crate) async fn insert_candidate_sentinel_id_key_pair(
    conn: &mut SqliteConnection,
    key_pair: &UnregisteredSentinelIdKeyPair,
    now: DateTime<Utc>,
) -> anyhow::Result<()> {
    let key_pair_json = serde_json::to_string(&key_pair.to_untrusted())?;

    sqlx::query!(
        r#"
            INSERT into candidate_sentinel_id_key_pair
            (key_pair_json, added_at)
            VALUES (?1, ?2)
        "#,
        key_pair_json,
        now
    )
    .execute(conn)
    .await?;

    Ok(())
}

pub(crate) async fn candidate_sentinel_id_key_pair(
    conn: &mut SqliteConnection,
) -> anyhow::Result<Option<CandidateSentinelIdKeyPairRow>> {
    let maybe_row = sqlx::query!(
        r#"
            SELECT
                candidate_sentinel_id_key_pair.id            AS "id: i64",
                candidate_sentinel_id_key_pair.key_pair_json AS "key_pair_json: String",
                candidate_sentinel_id_key_pair.added_at      AS "added_at: DateTime<Utc>"
            FROM candidate_sentinel_id_key_pair
        "#
    )
    .fetch_optional(conn)
    .await?
    .map(move |row| {
        let key_pair =
            serde_json::from_str::<UntrustedUnregisteredSentinelIdKeyPair>(&row.key_pair_json)?
                .to_trusted();

        let key_pair_row = CandidateSentinelIdKeyPairRow::new(row.id, row.added_at, key_pair);

        anyhow::Ok(key_pair_row)
    })
    .transpose()?;

    Ok(maybe_row)
}

pub(crate) async fn published_sentinel_id_key_pairs(
    conn: &mut SqliteConnection,
    now: DateTime<Utc>,
    trust_anchors: AnchorOrganizationPublicKeys,
) -> anyhow::Result<impl Iterator<Item = PublishedSentinelIdKeyPairRow>> {
    let org_pks_from_trust_anchors = trust_anchors.into_non_anchors();

    let key_pairs = sqlx::query!(
        r#"
            SELECT
                sentinel_id_key_pairs.id            AS "id: i64",
                sentinel_id_key_pairs.key_pair_json AS "key_pair_json: String",
                sentinel_id_key_pairs.published_at  AS "published_at: DateTime<Utc>",
                sentinel_id_key_pairs.epoch         AS "epoch: Epoch",
                journalist_provisioning_pks.pk_json AS "provisioning_pk_json: String"
            FROM sentinel_id_key_pairs
            JOIN journalist_provisioning_pks
                ON journalist_provisioning_pks.id = sentinel_id_key_pairs.provisioning_pk_id
        "#
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .flat_map(move |row| {
        let provisioning_pk = serde_json::from_str::<UntrustedJournalistProvisioningPublicKey>(
            &row.provisioning_pk_json,
        )?;

        // try to verify the provisioning pk against each trust anchor
        let provisioning_pk = org_pks_from_trust_anchors
            .iter()
            .find_map(|org_pk| {
                verify_journalist_provisioning_pk(&provisioning_pk, org_pk, now).ok()
            })
            .context(format!(
                "Could not verify provisioning pk for sentinel id key id {}",
                row.id
            ))?;

        let key_pair = serde_json::from_str::<UntrustedSentinelIdKeyPair>(&row.key_pair_json)?
            .to_trusted(&provisioning_pk, now)?;

        let key_pair_row = PublishedSentinelIdKeyPairRow::new(row.id, key_pair, row.epoch);

        anyhow::Ok(key_pair_row)
    });

    Ok(key_pairs)
}

pub(crate) async fn insert_registered_sentinel_id_key_pair(
    conn: &mut SqliteConnection,
    provisioning_pk_id: i64,
    key_pair: &SentinelIdKeyPair,
    created_at: DateTime<Utc>,
    published_at: DateTime<Utc>,
    epoch: Epoch,
) -> anyhow::Result<()> {
    let key_pair_json = serde_json::to_string(&key_pair.to_untrusted())?;

    sqlx::query!(
        r#"
            INSERT OR IGNORE INTO sentinel_id_key_pairs
                (provisioning_pk_id, key_pair_json, published_at, created_at, epoch)
            VALUES
                (?1, ?2, ?3, ?4, ?5)
        "#,
        provisioning_pk_id,
        key_pair_json,
        published_at,
        created_at,
        epoch,
    )
    .execute(conn)
    .await?;

    Ok(())
}

pub(crate) async fn delete_candidate_sentinel_id_key_pair(
    conn: &mut SqliteConnection,
    key_pair: &UnregisteredSentinelIdKeyPair,
) -> anyhow::Result<Option<CandidateSentinelIdKeyPairRow>> {
    let pk_json = serde_json::to_string(&key_pair.to_untrusted())?;

    let maybe_row = sqlx::query!(
        r#"
            DELETE FROM candidate_sentinel_id_key_pair
            WHERE json_extract(key_pair_json, '$.secret_key') = json_extract(?1, '$.secret_key')
            RETURNING
                id AS "id: i64",
                key_pair_json AS "key_pair_json: String",
                added_at AS "added_at: DateTime<Utc>"
        "#,
        pk_json
    )
    .fetch_optional(conn)
    .await?
    .map(|row| {
        let candidate_key_pair =
            serde_json::from_str::<UntrustedUnregisteredSentinelIdKeyPair>(&row.key_pair_json)?
                .to_trusted();

        anyhow::Ok(CandidateKeyPairRow::new(
            row.id,
            row.added_at,
            candidate_key_pair,
        ))
    })
    .transpose()?;

    Ok(maybe_row)
}

pub(crate) async fn last_published_sentinel_id_key_pair_at(
    conn: &mut SqliteConnection,
) -> anyhow::Result<Option<DateTime<Utc>>> {
    let row = sqlx::query!(
        r#"
            SELECT
                MAX(published_at) AS "published_at: DateTime<Utc>"
            FROM sentinel_id_key_pairs
        "#,
    )
    .fetch_one(conn)
    .await?;

    Ok(row.published_at)
}

pub(crate) async fn delete_expired_sentinel_id_key_pairs(
    conn: &mut SqliteConnection,
    now: DateTime<Utc>,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
            DELETE FROM sentinel_id_key_pairs
            WHERE key_pair_json->'public_key'->>'not_valid_after' < $1;
        "#,
        now
    )
    .execute(conn)
    .await?;

    Ok(())
}
