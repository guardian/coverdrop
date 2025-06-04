use chrono::{DateTime, Utc};
use common::{
    epoch::Epoch,
    protocol::keys::{
        anchor_org_pk, verify_journalist_provisioning_pk, JournalistIdKeyPair,
        UnregisteredJournalistIdKeyPair, UntrustedAnchorOrganizationPublicKey,
        UntrustedJournalistIdKeyPair, UntrustedJournalistProvisioningPublicKey,
        UntrustedUnregisteredJournalistIdKeyPair,
    },
};
use sqlx::SqliteConnection;

use crate::key_rows::{
    CandidateJournalistIdKeyPairRow, CandidateKeyPairRow, PublishedJournalistIdKeyPairRow,
};

pub(crate) async fn insert_candidate_id_key_pair(
    conn: &mut SqliteConnection,
    id_key_pair: &UnregisteredJournalistIdKeyPair,
    now: DateTime<Utc>,
) -> anyhow::Result<()> {
    let key_pair_json = serde_json::to_string(&id_key_pair.to_untrusted())?;

    sqlx::query!(
        r#"
            INSERT into candidate_journalist_id_key_pair
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

pub(crate) async fn candidate_id_key_pair(
    conn: &mut SqliteConnection,
) -> anyhow::Result<Option<CandidateJournalistIdKeyPairRow>> {
    let maybe_candidate_id_key_pair = sqlx::query!(
        r#"
            SELECT
                candidate_journalist_id_key_pair.id            AS "id: i64",
                candidate_journalist_id_key_pair.key_pair_json AS "id_key_pair_json: String",
                candidate_journalist_id_key_pair.added_at      AS "added_at: DateTime<Utc>"
            FROM candidate_journalist_id_key_pair
        "#
    )
    .fetch_optional(conn)
    .await?
    .map(move |row| {
        let id_key_pair_id = row.id;

        let id_key_pair = serde_json::from_str::<UntrustedUnregisteredJournalistIdKeyPair>(
            &row.id_key_pair_json,
        )?
        .to_trusted();

        let key_pair_row = CandidateJournalistIdKeyPairRow::new(id_key_pair_id, id_key_pair);

        anyhow::Ok(key_pair_row)
    })
    .transpose()?;

    Ok(maybe_candidate_id_key_pair)
}

pub(crate) async fn published_id_key_pairs(
    conn: &mut SqliteConnection,
    now: DateTime<Utc>,
) -> anyhow::Result<impl Iterator<Item = PublishedJournalistIdKeyPairRow>> {
    let id_key_pairs = sqlx::query!(
        r#"
            SELECT
                journalist_id_key_pairs.id            AS "id: i64",
                journalist_id_key_pairs.key_pair_json AS "id_key_pair_json: String",
                journalist_id_key_pairs.added_at      AS "added_at: DateTime<Utc>",
                journalist_id_key_pairs.epoch         AS "epoch: Epoch",
                journalist_provisioning_pks.pk_json   AS "provisioning_pk_json: String",
                anchor_organization_pks.pk_json      AS "org_pk_json: String"
            FROM journalist_id_key_pairs
            JOIN journalist_provisioning_pks
                ON journalist_provisioning_pks.id = journalist_id_key_pairs.provisioning_pk_id
            JOIN anchor_organization_pks
                ON anchor_organization_pks.id = journalist_provisioning_pks.organization_pk_id
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

        let id_key_pair_id = row.id;
        let epoch = row.epoch;

        let id_key_pair =
            serde_json::from_str::<UntrustedJournalistIdKeyPair>(&row.id_key_pair_json)?
                .to_trusted(&provisioning_pk, now)?;

        let key_pair_row = PublishedJournalistIdKeyPairRow::new(id_key_pair_id, id_key_pair, epoch);

        anyhow::Ok(key_pair_row)
    });

    Ok(id_key_pairs)
}

pub(crate) async fn insert_registered_id_key_pair(
    conn: &mut SqliteConnection,
    provisioning_pk_id: i64,
    id_key_pair: &JournalistIdKeyPair,
    epoch: Epoch,
    now: DateTime<Utc>,
) -> anyhow::Result<()> {
    let key_pair_json = serde_json::to_string(&id_key_pair.to_untrusted())?;

    sqlx::query!(
        r#"
            INSERT OR IGNORE INTO journalist_id_key_pairs
                (provisioning_pk_id, key_pair_json, added_at, epoch)
            VALUES
                (?1, ?2, ?3, ?4)
        "#,
        provisioning_pk_id,
        key_pair_json,
        now,
        epoch,
    )
    .execute(conn)
    .await?;

    Ok(())
}

pub(crate) async fn delete_candidate_id_key_pair(
    conn: &mut SqliteConnection,
    id_key_pair: &UnregisteredJournalistIdKeyPair,
) -> anyhow::Result<Option<CandidateJournalistIdKeyPairRow>> {
    let pk_json = serde_json::to_string(&id_key_pair.to_untrusted())?;

    let maybe_id = sqlx::query!(
        r#"
            DELETE FROM candidate_journalist_id_key_pair
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
            serde_json::from_str::<UntrustedUnregisteredJournalistIdKeyPair>(&row.key_pair_json)?
                .to_trusted();

        anyhow::Ok(CandidateKeyPairRow::new(row.id, candidate_key_pair))
    })
    .transpose()?;

    Ok(maybe_id)
}

pub(crate) async fn last_published_id_key_pair_at(
    conn: &mut SqliteConnection,
) -> anyhow::Result<Option<DateTime<Utc>>> {
    let row = sqlx::query!(
        r#"
            SELECT
                MAX(added_at) AS "added_at: DateTime<Utc>"
            FROM journalist_id_key_pairs
        "#,
    )
    .fetch_one(conn)
    .await?;

    Ok(row.added_at)
}

pub(crate) async fn delete_expired_id_key_pairs(
    conn: &mut SqliteConnection,
    now: DateTime<Utc>,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
            DELETE FROM journalist_id_key_pairs
            WHERE key_pair_json->'public_key'->>'not_valid_after' < $1;
        "#,
        now
    )
    .execute(conn)
    .await?;

    Ok(())
}
