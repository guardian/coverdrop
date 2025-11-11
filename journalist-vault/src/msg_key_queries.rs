use chrono::{DateTime, Utc};
use common::{
    epoch::Epoch,
    protocol::keys::{
        anchor_org_pk, verify_journalist_provisioning_pk, JournalistIdPublicKey,
        JournalistMessagingKeyPair, UntrustedAnchorOrganizationPublicKey,
        UntrustedJournalistIdKeyPair, UntrustedJournalistMessagingKeyPair,
        UntrustedJournalistProvisioningPublicKey,
    },
};
use sqlx::SqliteConnection;

use crate::{
    id_key_queries,
    key_rows::{CandidateJournalistMessagingKeyPairRow, PublishedJournalistMessagingKeyPairRow},
};

pub(crate) async fn candidate_msg_key_pair(
    conn: &mut SqliteConnection,
    now: DateTime<Utc>,
) -> anyhow::Result<Option<CandidateJournalistMessagingKeyPairRow>> {
    let maybe_candidate_msg_key_pair = sqlx::query!(
        r#"
            SELECT
                journalist_msg_key_pairs.id           AS "id: i64",
                journalist_msg_key_pairs.key_pair_json AS "msg_key_pair_json: String",
                journalist_msg_key_pairs.added_at     AS "added_at: DateTime<Utc>",
                journalist_id_key_pairs.key_pair_json  AS "id_key_pair_json: String",
                journalist_provisioning_pks.pk_json  AS "provisioning_pk_json: String",
                anchor_organization_pks.pk_json     AS "org_pk_json: String"
            FROM journalist_msg_key_pairs
            JOIN journalist_id_key_pairs
                ON journalist_id_key_pairs.id = journalist_msg_key_pairs.id_key_pair_id
            JOIN journalist_provisioning_pks
                ON journalist_provisioning_pks.id = journalist_id_key_pairs.provisioning_pk_id
            JOIN anchor_organization_pks
                ON anchor_organization_pks.id = journalist_provisioning_pks.organization_pk_id
            WHERE journalist_msg_key_pairs.epoch IS NULL
        "#
    )
    .fetch_optional(conn)
    .await?
    .map(move |row| {
        let org_pk =
            serde_json::from_str::<UntrustedAnchorOrganizationPublicKey>(&row.org_pk_json)?;
        let org_pk = anchor_org_pk(&org_pk, now)?.into_non_anchor();

        let provisioning_pk = serde_json::from_str::<UntrustedJournalistProvisioningPublicKey>(
            &row.provisioning_pk_json,
        )?;
        let provisioning_pk = verify_journalist_provisioning_pk(&provisioning_pk, &org_pk, now)?;

        let id_key_pair =
            serde_json::from_str::<UntrustedJournalistIdKeyPair>(&row.id_key_pair_json)?
                .to_trusted(&provisioning_pk, now)?;

        let msg_key_pair =
            serde_json::from_str::<UntrustedJournalistMessagingKeyPair>(&row.msg_key_pair_json)?
                .to_trusted(id_key_pair.public_key(), now)?;

        let id = row.id;
        let added_at = row.added_at;

        let key_pair_row = CandidateJournalistMessagingKeyPairRow::new(id, added_at, msg_key_pair);

        anyhow::Ok(key_pair_row)
    })
    .transpose()?;

    Ok(maybe_candidate_msg_key_pair)
}

pub(crate) async fn published_msg_key_pairs(
    conn: &mut SqliteConnection,
    now: DateTime<Utc>,
) -> anyhow::Result<impl Iterator<Item = PublishedJournalistMessagingKeyPairRow>> {
    let msg_key_pairs = sqlx::query!(
        r#"
            SELECT
                journalist_msg_key_pairs.id           AS "id: i64",
                journalist_msg_key_pairs.key_pair_json AS "msg_key_pair_json: String",
                journalist_msg_key_pairs.added_at     AS "added_at: DateTime<Utc>",
                journalist_msg_key_pairs.epoch        AS "epoch: Epoch",
                journalist_id_key_pairs.key_pair_json  AS "id_key_pair_json: String",
                journalist_provisioning_pks.pk_json  AS "provisioning_pk_json: String",
                anchor_organization_pks.pk_json     AS "org_pk_json: String"
            FROM journalist_msg_key_pairs
            JOIN journalist_id_key_pairs
                ON journalist_id_key_pairs.id = journalist_msg_key_pairs.id_key_pair_id
            JOIN journalist_provisioning_pks
                ON journalist_provisioning_pks.id = journalist_id_key_pairs.provisioning_pk_id
            JOIN anchor_organization_pks
                ON anchor_organization_pks.id = journalist_provisioning_pks.organization_pk_id
            WHERE journalist_msg_key_pairs.epoch IS NOT NULL
        "#
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .flat_map(move |row| {
        let org_pk =
            serde_json::from_str::<UntrustedAnchorOrganizationPublicKey>(&row.org_pk_json)?;
        let org_pk = anchor_org_pk(&org_pk, now)?.into_non_anchor();

        let provisioning_pk = serde_json::from_str::<UntrustedJournalistProvisioningPublicKey>(
            &row.provisioning_pk_json,
        )?;
        let provisioning_pk = verify_journalist_provisioning_pk(&provisioning_pk, &org_pk, now)?;

        let id_key_pair =
            serde_json::from_str::<UntrustedJournalistIdKeyPair>(&row.id_key_pair_json)?
                .to_trusted(&provisioning_pk, now)?;

        let msg_key_pair =
            serde_json::from_str::<UntrustedJournalistMessagingKeyPair>(&row.msg_key_pair_json)?
                .to_trusted(id_key_pair.public_key(), now)?;

        let id = row.id;

        let Some(epoch) = row.epoch else {
            tracing::error!("Found NULL epoch when fetching published messaging key pairs");

            anyhow::bail!("Invalid epoch value")
        };

        let key_pair_row = PublishedJournalistMessagingKeyPairRow::new(id, msg_key_pair, epoch);

        anyhow::Ok(key_pair_row)
    });

    Ok(msg_key_pairs)
}

pub(crate) async fn last_published_msg_key_pair_at(
    conn: &mut SqliteConnection,
) -> anyhow::Result<Option<DateTime<Utc>>> {
    let row = sqlx::query!(
        r#"
            SELECT
                MAX(added_at) AS "added_at: DateTime<Utc>"
            FROM journalist_msg_key_pairs
            WHERE epoch IS NOT NULL
        "#,
    )
    .fetch_one(conn)
    .await?;

    Ok(row.added_at)
}

pub(crate) async fn insert_candidate_msg_key_pair(
    conn: &mut SqliteConnection,
    id_pk: &JournalistIdPublicKey,
    msg_key_pair: &JournalistMessagingKeyPair,
    now: DateTime<Utc>,
) -> anyhow::Result<()> {
    let key_pair_json = serde_json::to_string(&msg_key_pair.to_untrusted())?;

    let id_key_pair_id = id_key_queries::published_id_key_pairs(conn, now)
        .await?
        .find(|key_pair_row| key_pair_row.key_pair.public_key() == id_pk)
        .map(|key_pair_row| key_pair_row.id)
        .ok_or_else(|| anyhow::anyhow!("Could not find the correct journalist ID key while inserting journalist messaging key pair"))?;

    sqlx::query!(
        r#"
            INSERT OR IGNORE INTO journalist_msg_key_pairs
                (id_key_pair_id, key_pair_json, added_at)
            VALUES
                (?1, ?2, ?3)
        "#,
        id_key_pair_id,
        key_pair_json,
        now,
    )
    .execute(conn)
    .await?;

    Ok(())
}

pub(crate) async fn promote_candidate_msg_key_pair_to_published(
    conn: &mut SqliteConnection,
    msg_key_pair: &JournalistMessagingKeyPair,
    epoch: Epoch,
) -> anyhow::Result<()> {
    let key_pair_json = serde_json::to_string(&msg_key_pair.to_untrusted())?;

    let result = sqlx::query!(
        r#"
            UPDATE journalist_msg_key_pairs
                SET epoch = ?1
            WHERE json_extract(key_pair_json, '$.secret_key')  = json_extract(?2, '$.secret_key')
        "#,
        epoch,
        key_pair_json
    )
    .execute(conn)
    .await?;

    if result.rows_affected() != 1 {
        tracing::warn!("Unexpected number of rows updated when promoting journalist messaging key pair from candidate to published, expected 1 was {}", result.rows_affected());
    }

    Ok(())
}

pub(crate) async fn delete_expired_msg_key_pairs(
    conn: &mut SqliteConnection,
    now: DateTime<Utc>,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
            DELETE FROM journalist_msg_key_pairs
            WHERE key_pair_json->'public_key'->>'not_valid_after' < $1;
        "#,
        now
    )
    .execute(conn)
    .await?;

    Ok(())
}
