use chrono::{DateTime, Utc};
use common::protocol::keys::{
    anchor_org_pk, verify_journalist_provisioning_pk, JournalistProvisioningPublicKey,
    OrganizationPublicKey, UntrustedAnchorOrganizationPublicKey,
    UntrustedJournalistProvisioningPublicKey,
};
use sqlx::SqliteConnection;

use crate::{key_rows::JournalistProvisioningPublicKeyRow, org_key_queries};

pub(crate) async fn journalist_provisioning_pks(
    conn: &mut SqliteConnection,
    now: DateTime<Utc>,
) -> anyhow::Result<impl Iterator<Item = JournalistProvisioningPublicKeyRow>> {
    let provisioning_pks = sqlx::query!(
        r#"
            SELECT
                journalist_provisioning_pks.id      AS "id: i64",
                journalist_provisioning_pks.pk_json AS "provisioning_pk_json: String",
                anchor_organization_pks.pk_json    AS "org_pk_json: String"
            FROM journalist_provisioning_pks
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

        let pk_row = JournalistProvisioningPublicKeyRow::new(row.id, provisioning_pk);

        anyhow::Ok(pk_row)
    });

    Ok(provisioning_pks)
}

pub(crate) async fn insert_journalist_provisioning_pk(
    conn: &mut SqliteConnection,
    org_pk: &OrganizationPublicKey,
    journalist_provisioning_pk: &JournalistProvisioningPublicKey,
    now: DateTime<Utc>,
) -> anyhow::Result<()> {
    let org_pk_id = org_key_queries::org_pks(conn, now)
        .await?
        .find(|db_org_pk| db_org_pk.pk == *org_pk)
        .map(|db_org_pk| db_org_pk.id)
        .ok_or_else(|| anyhow::anyhow!("Could not find the correct organization key while inserting journalist provisioning key"))?;

    let pk_json = serde_json::to_string(&journalist_provisioning_pk.to_untrusted())?;

    sqlx::query!(
        r#"
            INSERT OR IGNORE INTO journalist_provisioning_pks (organization_pk_id, pk_json, added_at)
            SELECT ?1, ?2, ?3
            WHERE NOT EXISTS (
                SELECT pk_json FROM journalist_provisioning_pks
                WHERE json_extract(pk_json, '$.key') = json_extract(?2, '$.key')
                AND json_extract(pk_json, '$.certificate') = json_extract(?2, '$.certificate')
                AND json_extract(pk_json, '$.not_valid_after') = json_extract(?2, '$.not_valid_after')
            );
        "#,
        org_pk_id,
        pk_json,
        now
    )
    .execute(conn)
    .await?;

    Ok(())
}

pub(crate) async fn journalist_provisioning_pk_id_from_pk(
    conn: &mut SqliteConnection,
    journalist_provisioning_pk: &JournalistProvisioningPublicKey,
) -> anyhow::Result<Option<i64>> {
    let pk_json = serde_json::to_string(&journalist_provisioning_pk.to_untrusted())?;

    let maybe_id = sqlx::query!(
        r#"
            SELECT id
            FROM journalist_provisioning_pks
            WHERE pk_json = ?1
        "#,
        pk_json
    )
    .fetch_optional(conn)
    .await?
    .map(|row| row.id);

    Ok(maybe_id)
}

pub(crate) async fn delete_expired_provisioning_pks(
    conn: &mut SqliteConnection,
    now: DateTime<Utc>,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
            DELETE FROM journalist_provisioning_pks
            WHERE pk_json->>'not_valid_after' < $1;
        "#,
        now
    )
    .execute(conn)
    .await?;

    Ok(())
}
